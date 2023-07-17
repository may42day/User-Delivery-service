use crate::schema::schema::users_queue;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable)]
#[diesel(table_name = users_queue)]
pub struct UserQueueInfo {
    pub id: i64,
    pub user_uuid: Uuid,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users_queue)]
pub struct AddUserToQueue {
    pub user_uuid: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = users_queue)]
pub struct UpdateQueueStatus {
    pub status: String,
}

#[derive(Queryable)]
#[diesel(table_name = users_queue)]
pub struct LastAttemp {
    pub id: i64,
    pub created_at: NaiveDateTime,
}
