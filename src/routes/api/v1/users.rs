use crate::handlers::users_handler::*;
use crate::middleware::jwt_middleware::JwtMiddleware;
use crate::middleware::logs_middleware::CustomRootSpanBuilder;
use crate::middleware::permissions_middleware::PermissionsMiddlewareFactory;
use crate::middleware::uuid_checker_middleware::UuidCheckerMiddlewareFactory;
use crate::utils::permission_policy::Policy;
use actix_web::web;
use tracing_actix_web::TracingLogger;

pub fn api_v1_users_config(cfg: &mut web::ServiceConfig, jwt_secret: String, policy: Policy) {
    let users_policy_mw = PermissionsMiddlewareFactory::new(policy.user_policy.clone());
    let admin_policy_mw = PermissionsMiddlewareFactory::new(policy.admin_policy.clone());
    let uuid_checker_mw = UuidCheckerMiddlewareFactory::default();
    let jwt_middleware = JwtMiddleware { jwt_secret };

    cfg.service(
        web::scope("api/v1/users")
            .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
            .wrap(jwt_middleware)
            .service(
                web::resource("/")
                    .route(web::get().to(get_all_users))
                    .wrap(admin_policy_mw.clone()),
            )
            .service(
                web::resource("/block/{uuid}")
                    .route(web::patch().to(block_user))
                    .wrap(admin_policy_mw),
            )
            .service(
                web::resource("/{uuid}")
                    .route(web::get().to(get_user_info))
                    .route(web::delete().to(delete_user))
                    .route(web::patch().to(update_user_profile))
                    .wrap(users_policy_mw.clone())
                    .wrap(uuid_checker_mw),
            )
            .service(
                web::resource("/me/")
                    .route(web::get().to(get_user_profile))
                    .wrap(users_policy_mw),
            ),
    );
}
