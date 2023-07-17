use crate::handlers::auth_handler::*;
use actix_web::web;

pub fn api_auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("auth").service(sign_in).service(sign_up));
}
