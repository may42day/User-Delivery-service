use crate::utils::errors::AppError;
use actix_web::web;
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};

pub async fn establish_connection_pool(database_url: &str) -> DbPool {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    Pool::builder()
        .build(config)
        .await
        .expect("Cannot build database pool")
}

pub type DbPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConn<'a> = bb8::PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn execute_connection<'a>(pool: &'a web::Data<DbPool>) -> Result<DbConn<'a>, AppError> {
    let db_conn = pool.get().await.map_err(AppError::db_error)?;
    Ok(db_conn)
}
