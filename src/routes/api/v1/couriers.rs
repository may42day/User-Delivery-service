use crate::handlers::couriers_handler::*;
use crate::middleware::jwt_middleware::JwtMiddleware;
use crate::middleware::logs_middleware::CustomRootSpanBuilder;
use crate::middleware::permissions_middleware::PermissionsMiddlewareFactory;
use crate::utils::permission_policy::Policy;
use actix_web::web;
use tracing_actix_web::TracingLogger;

pub fn api_v1_couriers_config(cfg: &mut web::ServiceConfig, jwt_secret: String, policy: Policy) {
    let courier_policy_mw = PermissionsMiddlewareFactory::new(policy.courier_policy.clone());
    let admin_policy_mw = PermissionsMiddlewareFactory::new(policy.admin_policy.clone());
    let jwt_middleware = JwtMiddleware { jwt_secret };

    cfg.service(
        web::scope("api/v1/couriers")
            .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
            .wrap(jwt_middleware)
            .service(
                web::resource("/")
                    .route(web::get().to(get_all_couriers))
                    .wrap(admin_policy_mw.clone()),
            )
            .service(
                web::resource("/{uuid}")
                    .route(web::patch().to(get_courier_info))
                    .wrap(admin_policy_mw),
            )
            .service(
                web::resource("/me/")
                    .route(web::get().to(get_courier_profile))
                    .wrap(courier_policy_mw),
            ),
    );
}
