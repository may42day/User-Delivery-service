use crate::schema::schema::users;
use crate::utils::validators::validate_role;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Queryable)]
pub struct Users {
    pub uuid: Uuid,
    pub first_name: String,
    pub address: Option<String>,
    pub phone_number: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub is_blocked: bool,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Clone, Validate)]
#[diesel(table_name = users)]
pub struct CreateUser {
    #[validate(length(
        min = 4,
        max = 30,
        message = "Name must be greater than 4 and less than 30 characters"
    ))]
    pub first_name: String,

    #[validate(phone)]
    pub phone_number: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 50,
        message = "Password must be greater than 8 and less than 50 characters long"
    ))]
    pub password: String,

    #[validate(custom(function = "validate_role", message = "Must contain USER or COURIER."))]
    pub role: String,
}

#[derive(Queryable, Insertable, Serialize)]
#[diesel(table_name = users)]
pub struct UserInfo {
    pub uuid: Uuid,
    pub first_name: String,
    pub address: Option<String>,
    pub phone_number: String,
    pub email: String,
    pub role: String,
    pub is_blocked: bool,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = users)]
pub struct UserProfile {
    pub first_name: String,
    pub address: Option<String>,
    pub phone_number: String,
    pub email: String,
}

#[derive(Queryable)]
#[diesel(table_name = users)]
pub struct UserTokenGeneratorInfo {
    pub uuid: Uuid,
    pub role: String,
}
