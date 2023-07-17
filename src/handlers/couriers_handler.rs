use crate::repository::couriers_repository;
use crate::resources::postgres::{execute_connection, DbPool};
use crate::services::auth_service::TokenClaims;
use crate::utils::errors::AppError;
use actix_web::web::ReqData;
use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;

pub async fn get_all_couriers(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let couriers = couriers_repository::select_all_couriers(&mut db_conn)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&couriers).map_err(AppError::serde_error)?))
}

pub async fn get_courier_info(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let uuid = path.into_inner();
    let courier = couriers_repository::select_courier(&mut db_conn, uuid)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&courier).map_err(AppError::serde_error)?))
}

pub async fn get_courier_profile(
    pool: web::Data<DbPool>,
    req_user: Option<ReqData<TokenClaims>>,
) -> Result<impl Responder, AppError> {
    let mut db_conn = execute_connection(&pool).await?;
    let uuid = req_user.unwrap().uuid;
    let user = couriers_repository::select_courier(&mut db_conn, uuid)
        .await
        .map_err(AppError::db_error)?;
    Ok(HttpResponse::Ok().body(serde_json::to_string(&user).map_err(AppError::serde_error)?))
}
