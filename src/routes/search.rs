use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::services::search_service::buscar_versiculos;
use crate::models::versiculo::Versiculo;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct SearchParams {
    pub word: String,
    pub page: usize,
    pub limit: usize,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub versiculos: Vec<Versiculo>,
    pub total: usize,
    pub total_pages: usize,
}

lazy_static! {
    static ref SEARCH_ATTEMPTS: Mutex<HashMap<String, (usize, Instant)>> = Mutex::new(HashMap::new());
}

const SEARCH_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60); // Janela de tempo de 60 segundos
const MAX_SEARCH_ATTEMPTS: usize = 50; // Limite de 25 buscas por janela

pub fn get_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search")
            .route("", web::get().to(search_bible)), // Rota GET para busca de versículos
    );
}

pub async fn search_bible(
    pool: web::Data<PgPool>,
    query: web::Query<SearchParams>,
    req: HttpRequest, // Para capturar o IP do cliente
) -> impl Responder {
    let word = query.word.trim(); // Remove espaços desnecessários
    let page = query.page;
    let limit = query.limit;

    // Captura o IP do cliente
    let ip_address = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();

    // Verificação de limite de requisições
    {
        let mut attempts = SEARCH_ATTEMPTS.lock().unwrap();
        let now = Instant::now();

        if let Some((count, timestamp)) = attempts.get_mut(&ip_address) {
            if now.duration_since(*timestamp) > SEARCH_RATE_LIMIT_WINDOW {
                *count = 1;
                *timestamp = now;
            } else {
                *count += 1;
                if *count > MAX_SEARCH_ATTEMPTS {
                    println!("IP {} excedeu o limite de buscas", ip_address);
                    return HttpResponse::TooManyRequests().json("Muitas buscas realizadas. Tente novamente mais tarde.");
                }
            }
        } else {
            attempts.insert(ip_address.clone(), (1, now));
        }
    }

    // Validação de entrada
    if word.is_empty() || word.len() > 100 {
        return HttpResponse::BadRequest().body("Palavra de busca inválida ou muito longa");
    }

    // let sanitized_word: String = word.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
    let sanitized_word: String = word.chars()
    .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "áéíóúâêîôûãõçÁÉÍÓÚÂÊÎÔÛÃÕÇ".contains(*c))
    .collect();



    if sanitized_word.is_empty() {
        return HttpResponse::BadRequest().body("Palavra de busca inválida após sanitização");
    }

    // Limitação do número máximo de itens por página
    let limit = limit.min(100); // Limita a 100 resultados por página para evitar abuso

    match buscar_versiculos(pool.get_ref(), &sanitized_word, page, limit).await {
        Ok((versiculos, total_versiculos, total_pages)) => {
            HttpResponse::Ok().json(SearchResponse {
                versiculos,
                total: total_versiculos,
                total_pages,
            })
        }
        Err(e) => {
            eprintln!("Erro ao buscar versículos: {:?}", e);
            HttpResponse::InternalServerError().body("Erro ao buscar versículos")
        }
    }
}