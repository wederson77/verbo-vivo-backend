use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use crate::services::jwt::verify_jwt;

pub async fn auth_middleware(req: HttpRequest) -> Result<String, Error> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];

                match verify_jwt(token) {
                    Ok(email) => return Ok(email),
                    Err(_) => return Err(actix_web::error::ErrorUnauthorized("Token inválido")),
                }
            }
        }
    }
    Err(actix_web::error::ErrorUnauthorized("Token não fornecido"))
}
