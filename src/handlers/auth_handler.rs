use crate::models::users_model::CreateUser;
use crate::resources::postgres::{execute_connection, DbPool};
use crate::services::{auth_service::*, users_service};
use crate::utils::configs::Config;
use crate::utils::errors::AppError;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::basic::BasicAuth;

#[post("/sign-in")]
async fn sign_in(
    pool: web::Data<DbPool>,
    credentials: BasicAuth,
    config: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let login = credentials.user_id().to_owned();
    let password = credentials.password();
    let config = &config.into_inner();

    match password {
        None => Ok(HttpResponse::Unauthorized().body("Password Required")),
        Some(password) => {
            let token_info =
                get_info_for_token(password.to_owned(), &mut db_conn, login, config).await?;
            let token = generate_token(token_info, config).await;

            Ok(HttpResponse::Ok().body(token))
        }
    }
}

#[post("/sign-up")]
async fn sign_up(
    data: actix_web_validator::Json<CreateUser>,
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let mut new_user = data.into_inner();
    let mut db_conn = execute_connection(&pool).await?;
    users_service::create_user(&mut new_user, &mut db_conn, &config).await?;
    Ok(HttpResponse::Created().body("{}"))
}
