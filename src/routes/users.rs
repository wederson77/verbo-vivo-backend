use actix_web::{web, Responder, HttpResponse};
use bcrypt::verify;
use crate::services::users::{create_user, find_user_by_email};
use crate::models::user::User;
use crate::services::jwt::generate_jwt; 
use crate::middlewaree::auth::auth_middleware;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::sync::Mutex;

// Estrutura para rastrear tentativas de login por IP
lazy_static! {
    static ref LOGIN_ATTEMPTS: Mutex<HashMap<String, (usize, Instant)>> = Mutex::new(HashMap::new());
}

const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60); // Janela de tempo: 60 segundos
const MAX_ATTEMPTS: usize = 5; // Máximo de tentativas permitidas

async fn user_info(req: actix_web::HttpRequest) -> impl Responder {
    match auth_middleware(req).await {
        Ok(email) => HttpResponse::Ok().json(serde_json::json!({ "email": email })),
        Err(err) => err.error_response(),
    }
}

pub fn get_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register_user))
       .route("/login", web::post().to(login_user));
}


// Função que lida com o cadastro de usuário

async fn register_user(pool: web::Data<PgPool>, user: web::Json<User>) -> impl Responder {
    println!("Requisição de cadastro recebida para o email: {}", user.email);

    match find_user_by_email(&pool, &user.email).await {
        Ok(Some(stored_password)) => {
            if verify(&user.password, &stored_password).unwrap_or(false) {
                println!("Usuário já registrado com esta senha: {}", user.email);
                return HttpResponse::BadRequest().json("Usuário já registrado com esta senha");
            } else {
                println!("Email já está em uso com outra senha: {}", user.email);
                return HttpResponse::Conflict().json("Email já está em uso com outra senha");
            }
        },
        Ok(None) => (),
        Err(err) => {
            println!("Erro ao verificar o usuário: {:?}", err);
            return HttpResponse::InternalServerError().json("Erro ao verificar o usuário");
        }
    }

    match create_user(&pool, &user.email, &user.password).await {
        Ok(_) => HttpResponse::Ok().json("Usuário registrado com sucesso"),
        Err(err) => {
            println!("Erro ao cadastrar o usuário: {:?}", err);
            HttpResponse::InternalServerError().json("Erro ao cadastrar o usuário")
        }
    }
}


// Função que lida com o login do usuário
async fn login_user(pool: web::Data<PgPool>, user: web::Json<User>, req: actix_web::HttpRequest) -> impl Responder {
    println!("Requisição de login recebida para o email: {}", user.email);

    let ip_address = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();

    {
        let mut attempts = LOGIN_ATTEMPTS.lock().unwrap();
        let now = Instant::now();

        if let Some((count, timestamp)) = attempts.get_mut(&ip_address) {
            if now.duration_since(*timestamp) > RATE_LIMIT_WINDOW {
                *count = 1;
                *timestamp = now;
            } else {
                *count += 1;
                if *count > MAX_ATTEMPTS {
                    println!("IP {} excedeu o limite de tentativas de login", ip_address);
                    return HttpResponse::TooManyRequests().json("Muitas tentativas de login. Tente novamente mais tarde.");
                }
            }
        } else {
            attempts.insert(ip_address.clone(), (1, now));
        }
    }

    match find_user_by_email(&pool, &user.email).await {
        Ok(Some(stored_password)) => {
            if verify(&user.password, &stored_password).unwrap_or(false) {
                println!("Login bem-sucedido para o email: {}", user.email);

                // Gerar um token JWT
                match generate_jwt(&user.email) {
                    Ok(token) => HttpResponse::Ok().json(serde_json::json!({ "token": token })),
                    Err(_) => HttpResponse::InternalServerError().json("Erro ao gerar token JWT"),
                }
            } else {
                println!("Senha incorreta para o email: {}", user.email);
                HttpResponse::Unauthorized().json("Senha incorreta")
            }
        },
        Ok(None) => {
            println!("Usuário não encontrado: {}", user.email);
            HttpResponse::NotFound().json("Usuário não encontrado")
        },
        Err(err) => {
            println!("Erro ao verificar o usuário: {:?}", err);
            HttpResponse::InternalServerError().json("Erro ao verificar o usuário")
        }
    }
}