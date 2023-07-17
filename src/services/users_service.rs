use crate::middleware::jwt_middleware::get_token_claims;
use crate::models::couriers_model::UpdateCourier;
use crate::models::queue_model::AddUserToQueue;
use crate::repository::couriers_repository::{find_free_courier, update_courier};
use crate::repository::queue_repository;
use crate::resources::postgres::DbPool;
use crate::services::auth_service::hash_password;
use crate::utils::configs::Config;
use crate::utils::errors::AppError;
use crate::utils::grpc::analytics_grpc::analytics_client::AnalyticsClient;
use crate::utils::grpc::users_grpc::users_server::Users;
use crate::utils::grpc::{analytics_grpc::*, users_grpc::*};
use crate::{
    models::{couriers_model::CreateCourier, users_model::CreateUser},
    repository::{couriers_repository, users_repository},
    resources::postgres::DbConn,
};
use chrono::{NaiveDateTime, Utc};
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

pub async fn create_user(
    new_user: &mut CreateUser,
    db_conn: &mut DbConn<'_>,
    config: &Config,
) -> Result<(), AppError> {
    new_user.password = hash_password(
        &new_user.password,
        &config.password_secret_key,
        &config.password_salt,
    )
    .await;

    let user = users_repository::create_user(db_conn, new_user.clone())
        .await
        .map_err(AppError::db_error)?;
    if user.role == *"COURIER" {
        let new_courier = CreateCourier {
            user_uuid: user.uuid,
        };
        let courier = couriers_repository::create_courier(new_courier, db_conn)
            .await
            .map_err(AppError::db_error)?;

        send_reg_info_to_analytics_service(config, user.uuid, "COURIER", courier.created_at)
            .await?;
        return Ok(());
    }

    send_reg_info_to_analytics_service(config, user.uuid, "USER", user.created_at).await?;
    Ok(())
}

async fn send_reg_info_to_analytics_service(
    config: &Config,
    uuid: Uuid,
    role: &str,
    created_at: NaiveDateTime,
) -> Result<(), AppError> {
    let mut client = AnalyticsClient::connect(config.grpc_analytics_address.clone())
        .await
        .map_err(AppError::grpc_error)?;
    let request = tonic::Request::new(SaveRegRequest {
        uuid: uuid.to_string(),
        role: role.to_owned(),
        created_at: created_at.to_string(),
    });
    client
        .save_reg_info(request)
        .await
        // .map_err(|e| AppError::grpc_error(e))?;
        .map_err(AppError::grpc_error)?;
    Ok(())
}

pub struct UserService {
    pub db_pool: DbPool,
    pub create_order_crone: i32,
    pub jwt_secret: String,
}

#[tonic::async_trait]
impl Users for UserService {
    async fn send_token_claims(
        &self,
        request: Request<TokenClaimsRequest>,
    ) -> Result<Response<TokenClaimsResponse>, Status> {
        let token = request.into_inner().token;
        let claims = get_token_claims(&token, &self.jwt_secret).await?;
        let response = TokenClaimsResponse {
            uuid: claims.uuid.to_string(),
            role: claims.role,
        };
        Ok(Response::new(response))
    }

    async fn find_courier(
        &self,
        request: Request<FindCourierRequest>,
    ) -> Result<Response<FindCourierResponse>, Status> {
        println!("inside fn");
        let mut db_conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
        println!("db pool created");
        let queue = queue_repository::select_unfinished_queue(&mut db_conn).await;
        println!("unfinished queue selected, handling");
        match queue {
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),

            // Find free courier and update his status if there is no queue
            Ok(queue) if queue.is_empty() => {
                println!("> empty queue");
                println!("searching for courier");
                let courier = find_free_courier(&mut db_conn)
                    .await
                    .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
                println!("handling courier");
                match courier {
                    // Sending courier and updating his status in case there is free courier
                    Some(courier) => {
                        let new_info = UpdateCourier {
                            is_free: Some(false),
                            rating: None,
                        };
                        update_courier(&mut db_conn, courier.user_uuid, new_info)
                            .await
                            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
                        let response = FindCourierResponse {
                            courier_uuid: courier.user_uuid.to_string(),
                            added_to_queue: false,
                            time_untill_next_try: 0,
                        };
                        Ok(Response::new(response))
                    }
                    // Adding user to queue in case there is no free couriers
                    None => {
                        println!("> none");
                        let request = request.into_inner();
                        let user = AddUserToQueue {
                            user_uuid: Uuid::parse_str(&request.user_uuid)
                                .expect("Cannot parse UUID"),
                        };
                        println!("> adding user to queue");
                        queue_repository::add_user_to_queue(&mut db_conn, user)
                            .await
                            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
                        println!("> making response");
                        let response = FindCourierResponse {
                            courier_uuid: "None".to_string(),
                            added_to_queue: true,
                            time_untill_next_try: 0,
                        };
                        Ok(Response::new(response))
                    }
                }
            }

            // In case queue isn't empty
            // Checking if user haven't been searching for courier before during adjusted time
            // Adding user in queue
            Ok(queue) => {
                println!("not empty queue");
                let request = request.into_inner();

                let uuid = Uuid::parse_str(&request.user_uuid).expect("Cannot parse UUID");
                let user = AddUserToQueue { user_uuid: uuid };
                // check if user already in queue
                for user in queue {
                    if user.user_uuid == uuid {
                        let response = FindCourierResponse {
                            courier_uuid: "None".to_string(),
                            added_to_queue: true,
                            time_untill_next_try: 0,
                        };
                        return Ok(Response::new(response));
                    }
                }

                println!("check time untill next attemp");
                let time =
                    time_untill_next_attemp(&mut db_conn, user.user_uuid, self.create_order_crone)
                        .await;
                if time > 0 {
                    let response = FindCourierResponse {
                        courier_uuid: "None".to_string(),
                        added_to_queue: false,
                        time_untill_next_try: time,
                    };
                    return Ok(Response::new(response));
                }
                println!("adding user to queue");
                queue_repository::add_user_to_queue(&mut db_conn, user)
                    .await
                    .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
                println!("making response");
                let response = FindCourierResponse {
                    courier_uuid: "None".to_string(),
                    added_to_queue: true,
                    time_untill_next_try: 0,
                };
                Ok(Response::new(response))
            }
        }
    }

    async fn update_courier_rating(
        &self,
        request: Request<UpdateCourierRatingRequest>,
    ) -> Result<Response<UpdateCourierRatingResponse>, Status> {
        let mut db_conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        let request = request.into_inner();
        let rating = &request.rating;
        let uuid = Uuid::parse_str(&request.courier_uuid).expect("Cannot parse Uuid");

        let new_info = UpdateCourier {
            is_free: None,
            rating: Some(*rating as f64),
        };
        update_courier(&mut db_conn, uuid, new_info)
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
        let response = UpdateCourierRatingResponse {
            message: "Updated".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn wait_for_courier(
        &self,
        request: Request<WaitForCourierRequest>,
    ) -> Result<Response<WaitForCourierResponse>, Status> {
        let mut db_conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        let request = request.into_inner();
        let uuid = Uuid::parse_str(&request.user_uuid).expect("Cannot parse Uuid");

        let queue = queue_repository::select_queue_info(&mut db_conn, uuid)
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
        match queue.status.as_ref() {
            "SEARCHING" => {
                let time = count_average_waiting_time(&mut db_conn, queue.id).await;
                let response = WaitForCourierResponse {
                    status: queue.status,
                    avg_waiting_time: time,
                };
                Ok(Response::new(response))
            }
            _ => {
                let response = WaitForCourierResponse {
                    status: queue.status,
                    avg_waiting_time: 0,
                };
                Ok(Response::new(response))
            }
        }
    }
}

// Count average waiting time for current user
// By default 180 sec just in case there are some problems with database
async fn count_average_waiting_time(db_conn: &mut DbConn<'_>, queue_id: i64) -> i32 {
    let default_time = 180;
    let completed_positions =
        queue_repository::select_last_ten_completed_positions(db_conn, queue_id).await;

    match completed_positions {
        Err(_) => default_time,
        // If there are no historical data
        // Result based on waiting time of first person in queue
        // and position of current user
        Ok(completed_positions) if completed_positions.is_empty() => {
            let current_queue =
                queue_repository::select_queue_untill_position(db_conn, queue_id).await;

            if let Ok(queue) = current_queue {
                let first_position = queue.first();
                if let Some(info) = first_position {
                    let first_person_time = Utc::now().naive_utc() - info.created_at;
                    let forecast_time =
                        first_person_time.num_seconds() as i32 * (queue.len() as i32);
                    println!("QUEUE LEN::::: {}", queue.len());
                    return forecast_time;
                }
            }
            default_time
        }

        // Using historical data to make forecast more accurate
        // Counting based on exponential smoothing
        Ok(completed_positions) => {
            let mut time_for_each_position: Vec<i32> =
                Vec::with_capacity(completed_positions.len());
            let current_time = Utc::now().naive_utc();
            let user_queue_created_at = completed_positions.last().unwrap().created_at;
            for position in completed_positions {
                time_for_each_position
                    .push((position.updated_at - position.created_at).num_seconds() as i32)
            }
            println!("TIME FOR EACH POSITION VEC: {:?}", time_for_each_position.clone());
            let average_time =
                time_for_each_position.iter().sum::<i32>() / (time_for_each_position.len() as i32);
                println!("AVERAGE TIME: {:?}", average_time);
            // Adding to forecast time of current persons in queue
            // if their waiting time already more than average time
            if let Ok(queue) =
                queue_repository::select_queue_untill_position(db_conn, queue_id).await
            {
                for position in queue {
                    let pos_time = (current_time - position.created_at).num_seconds() as i32;

                    if pos_time > average_time {
                        time_for_each_position.push(pos_time)
                    } else {
                        break;
                    }
                }
            };

            // ALPHA is the smoothing parameter that defines the weighting
            // and should be greater than 0 and less than 1
            let alpha = 0.2;
            let mut exponential_smoothing = vec![*time_for_each_position.first().unwrap() as f64];
            for i in time_for_each_position.iter().map(|n| *n as f64).skip(1) {
                let smoothed_value =
                    alpha * i + (1.0 - alpha) * exponential_smoothing.last().unwrap();
                exponential_smoothing.push(smoothed_value)
            }

            let smoothing_value = *exponential_smoothing.last().unwrap() as i32;
            let sec_left = smoothing_value - (current_time - user_queue_created_at).num_seconds() as i32;
            if sec_left > 0 {
                sec_left
            } else { 
                smoothing_value
            }

        }
    }
}

async fn time_untill_next_attemp(
    db_conn: &mut DbConn<'_>,
    uuid: Uuid,
    create_order_crone: i32,
) -> i32 {
    let last_attemp = queue_repository::get_last_user_try(db_conn, uuid).await;
    if let Ok(time) = last_attemp {
        let current_time = Utc::now().naive_utc();
        if (current_time - time.created_at).num_seconds() > create_order_crone as i64 {
            return (time.created_at - current_time).num_seconds() as i32 + create_order_crone;
        }
    }
    0
}
