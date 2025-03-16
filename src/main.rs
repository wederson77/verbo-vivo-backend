use actix_web::{middleware, App, HttpServer, web};
use actix_cors::Cors;
use sqlx::PgPool;
use dotenvy::dotenv;
use crate::routes::configure_routes;
use sqlx::postgres::PgPoolOptions;
use num_cpus;
use env_logger;
use log::info;
use std::env;
use std::time::Duration;

mod routes;
mod services;
mod models;
mod middlewaree;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init(); // Inicializa o logger

    // Obtendo a URL do banco de dados do .env
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL não configurado");

    let max_connections = env::var("PG_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string()) // Valor padrão caso não seja definido
        .parse::<u32>()
        .expect("PG_MAX_CONNECTIONS deve ser um número válido");

    let min_connections = env::var("PG_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<u32>()
        .expect("PG_MIN_CONNECTIONS deve ser um número válido");

    let idle_timeout = env::var("PG_IDLE_TIMEOUT")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .expect("PG_IDLE_TIMEOUT deve ser um número válido");

    // Criando pool de conexões do banco
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .idle_timeout(Duration::from_secs(idle_timeout))
        .connect(&database_url)
        .await
        .expect("Falha ao conectar ao banco de dados");

    info!("Conectado ao banco de dados com sucesso!");

    // Obtendo a porta definida pelo Render
    let port = env::var("PORT")
        .unwrap_or_else(|_| "4000".to_string()) // Se não estiver definida, usa 4000
        .parse::<u16>()
        .expect("PORT deve ser um número válido");

    // Configurando o número de workers
    let num_workers = env::var("WORKERS")
        .unwrap_or_else(|_| (num_cpus::get() / 2).to_string()) // Usa metade dos núcleos da CPU
        .parse::<usize>()
        .expect("WORKERS deve ser um número válido");

    info!("Iniciando servidor na porta {}", port);

    // Iniciando o servidor
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .supports_credentials()
                .max_age(3600))
            .configure(configure_routes)
    })
    .workers(num_workers)
    .bind(("0.0.0.0", port))? // Ouvindo em 0.0.0.0 para funcionar no Render
    .run()
    .await
}
