use crate::schema::schema::couriers;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable)]
#[diesel(belongs_to(Users))]
#[diesel(table_name = couriers)]
pub struct Couriers {
    pub user_uuid: Uuid,
    pub is_free: bool,
    pub rating: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = couriers)]
pub struct CreateCourier {
    pub user_uuid: Uuid,
}

#[derive(Serialize, Queryable)]
#[diesel(table_name = couriers)]
pub struct CourierInfo {
    pub user_uuid: Uuid,
    pub is_free: bool,
    pub rating: f64,
}

#[derive(AsChangeset)]
#[diesel(table_name = couriers)]
pub struct UpdateCourier {
    pub is_free: Option<bool>,
    pub rating: Option<f64>,
}
