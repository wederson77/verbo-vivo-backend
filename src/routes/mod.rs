use actix_web::{web, Scope};
pub mod search;
pub mod users; 

use crate::routes::users::get_routes as user_routes;
use crate::routes::search::get_routes as search_routes; 
use crate::middlewaree::auth::auth_middleware as user_info;



pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    user_routes(cfg);
    search_routes(cfg);
    cfg.route("/me", web::get().to(user_info));
}