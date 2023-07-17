use crate::models::couriers_model::CourierInfo;
use crate::resources::postgres::DbConn;
use crate::utils::grpc::orders_grpc::orders_client::OrdersClient;
use crate::utils::grpc::orders_grpc::{
    CourierForUserRequest, CourierForUserResponse, TimeExpirationRequest, TimeExpirationResponse,
};
use crate::{
    models::couriers_model::UpdateCourier,
    repository::{
        couriers_repository::{find_free_courier, update_courier},
        queue_repository::{self, change_order_status},
    },
    utils::configs::Config,
};
use chrono::Utc;
use std::{thread, time::Duration};
use tonic::{Response, Status};
use tracing::{error, info};
use uuid::Uuid;

pub async fn courier_distribution_loop(
    config: Config,
    mut db_conn: DbConn<'_>,
) -> Result<(), anyhow::Error> {
    loop {
        let queue = queue_repository::select_unfinished_queue(&mut db_conn).await;
        match queue {
            Err(e) => {
                error!("Error selectig unfinished queue {e}");
            }
            // Wait a few seconds before next queue checking
            // in case queue is empty
            Ok(queue) if queue.is_empty() => thread::sleep(Duration::from_secs(2)),
            Ok(queue) => {
                // Checking for orders with expired time and notify order service about that event
                for user in &queue {
                    let is_expired = (Utc::now().naive_utc() - user.created_at).num_seconds()
                        > config.order_max_waiting_time.into();
                    if is_expired {
                        let status_changed =
                            change_order_status(&mut db_conn, user.id, "EXPIRED").await;
                        if status_changed.is_ok() {
                            let _ = note_user_about_time_expiration(&config, user.user_uuid).await;
                        }
                    }
                }

                // Finding courier for first person in queue and sending notification to order service
                let first_in_queue = queue.first().expect("It cannot be empty because of previous checking");

                // searching for couriers untill find one or untill time expiration
                'first_in_queue: loop {
                    let is_expired = (Utc::now().naive_utc() - first_in_queue.created_at)
                        .num_seconds()
                        > config.order_max_waiting_time.into();
                    if is_expired {
                        let status_changed =
                            change_order_status(&mut db_conn, first_in_queue.id, "EXPIRED").await;
                        if status_changed.is_ok() {
                            let _ = note_user_about_time_expiration(&config, first_in_queue.user_uuid).await;
                        }
                        break 'first_in_queue;
                    }

                    let courier = find_free_courier(&mut db_conn).await;
                    if let Ok(Some(courier)) = courier {
                        let status_changed =
                            change_order_status(&mut db_conn, first_in_queue.id, "COMPLETED").await;
                        if status_changed.is_ok() {
                            let info = UpdateCourier {
                                is_free: Some(false),
                                rating: None,
                            };
                            let courier_status_changed =
                                update_courier(&mut db_conn, courier.user_uuid, info).await;
                            if courier_status_changed.is_ok() {
                                let user_noted = note_user_about_founded_courier(
                                    &config,
                                    first_in_queue.user_uuid,
                                    &courier,
                                )
                                .await;
                                if let Err(e) = user_noted {
                                    error!("Error sending notification to order service, {e}");
                                }
                            }
                        }

                        break 'first_in_queue;
                    }
                }
            }
        }
    }
}

async fn note_user_about_time_expiration(
    config: &Config,
    uuid_user: Uuid,
) -> Result<Response<TimeExpirationResponse>, Status> {
    let connected = OrdersClient::connect(config.grpc_orders_address.to_owned()).await;
    match connected {
        Ok(mut client) => {
            let request = tonic::Request::new(TimeExpirationRequest {
                user_uuid: uuid_user.to_string(),
            });
            Ok(client.notify_expiration_time(request).await?)
        }
        Err(_) => Err(Status::internal("Internal error")),
    }
}

async fn note_user_about_founded_courier(
    config: &Config,
    uuid_user: Uuid,
    courier: &CourierInfo,
) -> Result<Response<CourierForUserResponse>, Status> {
    let connected = OrdersClient::connect(config.grpc_analytics_address.to_owned()).await;
    match connected {
        Ok(mut client) => {
            let request = tonic::Request::new(CourierForUserRequest {
                courier_uuid: courier.user_uuid.to_string(),
                user_uuid: uuid_user.to_string(),
                courier_rating: courier.rating as f32,
            });
            Ok(client.notify_founded_courier(request).await?)
        }
        Err(e) => Err(Status::internal(format!("Internal error: {}", e))),
    }
}

// Trying to connect to Orders gRPC server until success
pub async fn check_grpc_connection(config: &Config) {
    while OrdersClient::connect(config.grpc_orders_address.to_owned())
        .await
        .is_err()
    {
        info!("Cannot connect to Orders gRPC server. Trying to reconnect ...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    info!("Connected to Order gRPC server");
}
