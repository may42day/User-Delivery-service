use crate::routes::api::auth::auth::api_auth_config;
use crate::routes::api::v1::v1_config::api_v1_config;
use crate::utils::permission_policy::Policy;
use actix_web::web;

pub fn api_config(cfg: &mut web::ServiceConfig, jwt_secret: String, policy: Policy) {
    cfg.configure(api_auth_config);
    cfg.configure(move |cfg| api_v1_config(cfg, jwt_secret, policy));
}
