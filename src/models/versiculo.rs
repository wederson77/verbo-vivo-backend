use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Versiculo {
    pub id: i32,        // ID do versículo
    pub livro: String,  // Nome do livro
    pub texto: String,  // Texto do versículo
}