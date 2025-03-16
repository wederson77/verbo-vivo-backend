use crate::models::versiculo::Versiculo;
use sqlx::PgPool;
use anyhow::Result;

/// Busca os versículos contendo uma palavra específica no banco de dados
pub async fn buscar_versiculos(
    pool: &PgPool,
    word: &str,
    page: usize,
    limit: usize,
) -> Result<(Vec<Versiculo>, usize, usize)> {
    // Cálculo de paginação
    let offset = (page - 1) * limit;

    // Criação da palavra de busca para full-text search
    let search_word = format!("{}:*", word.to_lowercase()); // Formato para busca em texto completo

    // Consulta para contar o total de versículos que correspondem à pesquisa
    let total_versiculos: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) 
         FROM versiculos 
         WHERE to_tsvector('portuguese', texto) @@ to_tsquery('portuguese', $1)",
    )
    .bind(&search_word)
    .fetch_one(pool)
    .await?;

    let total_versiculos = total_versiculos.0 as usize;
    let total_pages = (total_versiculos + limit - 1) / limit;

    // Consulta para buscar os versículos com paginação
    let versiculos = sqlx::query_as::<_, Versiculo>(
        "SELECT id, livro, texto 
         FROM versiculos 
         WHERE to_tsvector('portuguese', texto) @@ to_tsquery('portuguese', $1)
         ORDER BY id
         LIMIT $2 OFFSET $3",
    )
    .bind(&search_word)
    .bind(limit as i32)
    .bind(offset as i32)
    .fetch_all(pool)
    .await?;

    Ok((versiculos, total_versiculos, total_pages))
}