use crate::models::users_model::*;
use crate::resources::postgres::DbConn;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn create_user(db_conn: &mut DbConn<'_>, user: CreateUser) -> Result<Users, Error> {
    use crate::schema::schema::users::dsl::*;
    diesel::insert_into(users)
        .values(user)
        .get_result(db_conn)
        .await
}

pub async fn select_all_users(db_conn: &mut DbConn<'_>) -> Result<Vec<UserInfo>, Error> {
    use crate::schema::schema::users::dsl::*;
    users
        .select((
            uuid,
            first_name,
            address,
            phone_number,
            email,
            role,
            is_blocked,
            is_deleted,
            created_at,
        ))
        .load::<UserInfo>(db_conn)
        .await
}

pub async fn select_user(db_conn: &mut DbConn<'_>, user_uuid: Uuid) -> Result<UserInfo, Error> {
    use crate::schema::schema::users::dsl::*;
    users
        .filter(uuid.eq(user_uuid))
        .select((
            uuid,
            first_name,
            address,
            phone_number,
            email,
            role,
            is_blocked,
            is_deleted,
            created_at,
        ))
        .get_result::<UserInfo>(db_conn)
        .await
}

pub async fn delete_user(db_conn: &mut DbConn<'_>, user_uuid: Uuid) -> Result<usize, Error> {
    use crate::schema::schema::users::dsl::*;
    diesel::update(users.find(user_uuid))
        .set((is_deleted.eq(true),))
        .execute(db_conn)
        .await
}

pub async fn update_profile(
    db_conn: &mut DbConn<'_>,
    user_uuid: Uuid,
    user: UserProfile,
) -> Result<usize, Error> {
    use crate::schema::schema::users::dsl::*;
    diesel::update(users.find(user_uuid))
        .set(&user)
        .execute(db_conn)
        .await
}

pub async fn select_user_info_for_token(
    db_conn: &mut DbConn<'_>,
    phone: String,
    hashed_password: String,
) -> Result<UserTokenGeneratorInfo, Error> {
    use crate::schema::schema::users::dsl::*;
    users
        .filter(phone_number.eq(phone).and(password.eq(hashed_password)))
        .select((uuid, role))
        .get_result::<UserTokenGeneratorInfo>(db_conn)
        .await
}

pub async fn block_user(db_conn: &mut DbConn<'_>, user_uuid: Uuid) -> Result<usize, Error> {
    use crate::schema::schema::users::dsl::*;
    diesel::update(users.find(user_uuid))
        .set((is_blocked.eq(true),))
        .execute(db_conn)
        .await
}
