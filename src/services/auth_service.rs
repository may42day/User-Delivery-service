use crate::{
    models::users_model::UserTokenGeneratorInfo,
    repository::users_repository,
    resources::postgres::DbConn,
    utils::{configs::Config, errors::AppError},
};
use argonautica::Hasher;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenClaims {
    pub uuid: Uuid,
    pub role: String,
}

pub async fn hash_jwt_secret(jwt_secret: &str) -> Hmac<Sha256> {
    Hmac::new_from_slice(jwt_secret.as_bytes()).expect("HMAC can take key of any size")
}

pub async fn hash_password(
    password: &str,
    password_secret_key: &str,
    password_salt: &str,
) -> String {
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password(password)
        .with_secret_key(password_secret_key)
        .with_salt(password_salt)
        .hash()
        .expect("Cannot hash passwrod");

    hash
}

pub async fn get_info_for_token(
    password: String,
    db_conn: &mut DbConn<'_>,
    login: String,
    config: &Config,
) -> Result<UserTokenGeneratorInfo, AppError> {
    let hashed_password = hash_password(
        &password,
        &config.password_secret_key,
        &config.password_salt,
    )
    .await;
    let info_for_token =
        users_repository::select_user_info_for_token(db_conn, login, hashed_password)
            .await
            .map_err(AppError::db_error)?;
    Ok(info_for_token)
}

pub async fn generate_token(info_for_token: UserTokenGeneratorInfo, config: &Config) -> String {
    let claims = TokenClaims {
        uuid: info_for_token.uuid,
        role: info_for_token.role,
    };
    let jwt_secret: Hmac<Sha256> = hash_jwt_secret(&config.jwt_secret).await;
    claims
        .sign_with_key(&jwt_secret)
        .expect("Cannot sign object with a key")
}
