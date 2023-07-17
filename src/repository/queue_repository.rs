use crate::models::queue_model::*;
use crate::resources::postgres::DbConn;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub async fn select_unfinished_queue(
    db_conn: &mut DbConn<'_>,
) -> Result<Vec<UserQueueInfo>, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    users_queue
        .filter(status.eq("SEARCHING"))
        .select((id, user_uuid, status, created_at, updated_at))
        .get_results::<UserQueueInfo>(db_conn)
        .await
}

pub async fn select_queue_info(
    db_conn: &mut DbConn<'_>,
    uuid: Uuid,
) -> Result<UserQueueInfo, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    users_queue
        .filter(user_uuid.eq(uuid))
        .order(created_at.desc())
        .limit(1)
        .select((id, user_uuid, status, created_at, updated_at))
        .get_result::<UserQueueInfo>(db_conn)
        .await
}

pub async fn select_queue_untill_position(
    db_conn: &mut DbConn<'_>,
    queue_id: i64,
) -> Result<Vec<UserQueueInfo>, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    users_queue
        .filter(status.eq("SEARCHING").and(id.lt(queue_id)))
        .select((id, user_uuid, status, created_at, updated_at))
        .get_results::<UserQueueInfo>(db_conn)
        .await
}

pub async fn select_last_ten_completed_positions(
    db_conn: &mut DbConn<'_>,
    queue_id: i64,
) -> Result<Vec<UserQueueInfo>, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    users_queue
        .order(created_at.desc())
        .filter(status.eq("COMPLETED").and(id.le(queue_id)))
        .select((id, user_uuid, status, created_at, updated_at))
        .limit(10)
        .get_results::<UserQueueInfo>(db_conn)
        .await
}

pub async fn add_user_to_queue(
    db_conn: &mut DbConn<'_>,
    user: AddUserToQueue,
) -> Result<usize, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    diesel::insert_into(users_queue)
        .values(user)
        .execute(db_conn)
        .await
}

pub async fn change_order_status(
    db_conn: &mut DbConn<'_>,
    queue_id: i64,
    new_status: &str,
) -> Result<usize, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    diesel::update(users_queue.find(queue_id))
        .set(status.eq(new_status))
        .execute(db_conn)
        .await
}

pub async fn get_last_user_try(db_conn: &mut DbConn<'_>, user: Uuid) -> Result<LastAttemp, Error> {
    use crate::schema::schema::users_queue::dsl::*;
    users_queue
        .filter(user_uuid.eq(user))
        .order(created_at.desc())
        .limit(1)
        .select((id, updated_at))
        .get_result::<LastAttemp>(db_conn)
        .await
}
