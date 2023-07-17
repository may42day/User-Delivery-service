use crate::models::couriers_model::*;
use crate::resources::postgres::DbConn;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn create_courier(
    courier: CreateCourier,
    db_conn: &mut DbConn<'_>,
) -> Result<Couriers, Error> {
    use crate::schema::schema::couriers::dsl::*;

    diesel::insert_into(couriers)
        .values(courier)
        .get_result(db_conn)
        .await
}

pub async fn find_free_courier(db_conn: &mut DbConn<'_>) -> Result<Option<CourierInfo>, Error> {
    use crate::schema::schema::couriers::dsl::*;

    couriers
        .filter(is_free.eq(true))
        .select((user_uuid, is_free, rating))
        .limit(1)
        .get_result(db_conn)
        .await
        .optional()
}

pub async fn update_courier(
    db_conn: &mut DbConn<'_>,
    uuid: Uuid,
    new_info: UpdateCourier,
) -> Result<usize, Error> {
    use crate::schema::schema::couriers::dsl::*;

    diesel::update(couriers)
        .filter(user_uuid.eq(uuid))
        .set(new_info)
        .execute(db_conn)
        .await
}

pub async fn select_courier(db_conn: &mut DbConn<'_>, uuid: Uuid) -> Result<CourierInfo, Error> {
    use crate::schema::schema::couriers::dsl::*;
    couriers
        .filter(user_uuid.eq(uuid))
        .select((user_uuid, is_free, rating))
        .get_result::<CourierInfo>(db_conn)
        .await
}

pub async fn select_all_couriers(db_conn: &mut DbConn<'_>) -> Result<Vec<CourierInfo>, Error> {
    use crate::schema::schema::couriers::dsl::*;
    couriers
        .select((user_uuid, is_free, rating))
        .load::<CourierInfo>(db_conn)
        .await
}
