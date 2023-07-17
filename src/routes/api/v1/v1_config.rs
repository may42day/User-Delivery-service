use crate::{
    routes::api::v1::{couriers, users},
    utils::permission_policy::Policy,
};
use actix_web::web;

pub fn api_v1_config(cfg: &mut web::ServiceConfig, jwt_secret: String, policy: Policy) {
    let cloned_jwt_secret = jwt_secret.clone();
    let cloned_policy = policy.clone();
    cfg.configure(move |cfg| users::api_v1_users_config(cfg, cloned_jwt_secret, cloned_policy));
    cfg.configure(move |cfg| couriers::api_v1_couriers_config(cfg, jwt_secret, policy));
}
