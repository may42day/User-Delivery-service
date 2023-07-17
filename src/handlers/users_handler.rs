use crate::models::users_model::*;
use crate::repository::users_repository;
use crate::resources::postgres::{execute_connection, DbPool};
use crate::services::auth_service::TokenClaims;
use crate::utils::errors::AppError;
use actix_web::web::ReqData;
use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;

pub async fn get_all_users(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let users = users_repository::select_all_users(&mut db_conn)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&users).map_err(AppError::serde_error)?))
}

pub async fn get_user_profile(
    pool: web::Data<DbPool>,
    req_user: Option<ReqData<TokenClaims>>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let uuid = req_user.unwrap().uuid;
    let user = users_repository::select_user(&mut db_conn, uuid)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&user).map_err(AppError::serde_error)?))
}

pub async fn get_user_info(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let uuid = path.into_inner();
    let user = users_repository::select_user(&mut db_conn, uuid)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&user).map_err(AppError::serde_error)?))
}

pub async fn block_user(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let uuid = path.into_inner();
    users_repository::block_user(&mut db_conn, uuid)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body("User blocked"))
}

pub async fn update_user_profile(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    data: web::Json<UserProfile>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let user_profile = data.into_inner();
    let uuid = path.into_inner();
    users_repository::update_profile(&mut db_conn, uuid, user_profile)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body("{}"))
}

pub async fn delete_user(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let uuid = path.into_inner();
    users_repository::delete_user(&mut db_conn, uuid)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body("User deleted"))
}
