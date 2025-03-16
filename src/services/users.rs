use bcrypt::{verify, hash, DEFAULT_COST};
use sqlx::PgPool;
use anyhow::Result;
use crate::models::user::User;
use std::sync::Mutex;
use lazy_static::lazy_static;


#[derive(sqlx::FromRow)]
struct UserRow {
    password: String,
}

// Função para criar usuário com senha criptografada
pub async fn create_user(pool: &PgPool, email: &str, password: &str) -> Result<()> {
    // Gerando hash da senha
    let hashed_password = hash(password, DEFAULT_COST)?;
    
    // Inserindo usuário no banco
    sqlx::query!(
        "INSERT INTO users (email, password) VALUES ($1, $2)",
        email,
        hashed_password
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

// Função para encontrar usuário por email e retornar a senha criptografada
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<String>> {
    let record = sqlx::query_as::<_, UserRow>(
        "SELECT password FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    
    Ok(record.map(|r| r.password))
}