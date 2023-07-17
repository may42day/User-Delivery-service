use crate::middleware::logs_middleware::CustomRootSpanBuilder;
use crate::routes::api::config;
use crate::services::couriers_service::{check_grpc_connection, courier_distribution_loop};
use crate::services::users_service::UserService;
use crate::utils::grpc::users_grpc::users_server::UsersServer;
use crate::{
    middleware::logs_middleware::init_tracing_suscriber,
    resources::postgres::{establish_connection_pool, DbPool},
};
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use structopt::StructOpt;
use tonic::transport::server::Router;
use tonic::transport::Server;
use tracing::info;
use tracing_actix_web::TracingLogger;
use super::permission_policy::Policy;

#[derive(Debug, StructOpt, Clone)]
pub struct Opt {
    #[structopt(long, env = "JWT_SECRET")]
    pub jwt_secret: String,

    #[structopt(long, env = "PASSWORD_SALT", default_value = "password_salt")]
    pub password_salt: String,

    #[structopt(long, env = "PASSWORD_SECRET_KEY", default_value = "some_password_key")]
    pub password_secret_key: String,

    #[structopt(long, env = "DATABASE_URL")]
    pub database_url: String,

    #[structopt(long, env = "ORDER_MAX_WAITING_TIME", default_value = "180")]
    pub order_max_waiting_time: i32,

    #[structopt(long, env = "CREATE_ORDER_CRONE", default_value = "300")]
    pub create_order_crone: i32,

    #[structopt(long, env = "BIND_ADDRESS", default_value = "0.0.0.0:8080")]
    pub bind_address: String,

    #[structopt(long, env = "GRPC_USER_ADDRESS", default_value = "0.0.0.0:50051")]
    pub grpc_users_address: String,

    #[structopt(
        long,
        env = "GRPC_ORDERS_ADDRESS",
        default_value = "http://0.0.0.0:50052"
    )]
    pub grpc_orders_address: String,

    #[structopt(
        long,
        env = "GRPC_ANALYTICS_ADDRESS",
        default_value = "http://0.0.0.0:50053"
    )]
    pub grpc_analytics_address: String,
}

#[derive(Clone)]
pub struct Config {
    pub permission_policy: Policy,
    pub jwt_secret: String,
    pub password_salt: String,
    pub password_secret_key: String,
    pub db_pool: DbPool,
    pub order_max_waiting_time: i32,
    pub create_order_crone: i32,
    pub bind_address: String,
    pub grpc_users_address: String,
    pub grpc_orders_address: String,
    pub grpc_analytics_address: String,
}

impl Config {
    pub async fn init() -> Config {
        dotenv().ok();
        init_tracing_suscriber().await;
        let opt = Opt::from_args();

        let permission_policy = Policy::build();
        let jwt_secret = opt.jwt_secret;
        let password_salt = opt.password_salt;
        let password_secret_key = opt.password_secret_key;
        let db_pool = establish_connection_pool(&opt.database_url).await;
        let order_max_waiting_time = opt.order_max_waiting_time;
        let create_order_crone = opt.create_order_crone;
        let bind_address = opt.bind_address;
        let grpc_users_address = opt.grpc_users_address;
        let grpc_orders_address = opt.grpc_orders_address;
        let grpc_analytics_address = opt.grpc_analytics_address;

        Config {
            permission_policy,
            jwt_secret,
            password_salt,
            password_secret_key,
            db_pool,
            order_max_waiting_time,
            create_order_crone,
            bind_address,
            grpc_users_address,
            grpc_orders_address,
            grpc_analytics_address,
        }
    }
}

pub async fn run_courier_distributor_untill_stopped(config: Config) -> Result<(), anyhow::Error> {
    info!("Starting courier distribution handler.");
    check_grpc_connection(&config).await;
    let db_pool = config.db_pool.clone();
    let db_conn = db_pool.get().await?;
    courier_distribution_loop(config, db_conn).await
}

pub struct JwtSecret {
    pub jwt: String,
}

pub struct Application {
    server: actix_web::dev::Server,
}

impl Application {
    pub async fn build(config: &Config) -> Result<Self, anyhow::Error> {
        info!("Building application");
        let config = config.clone();

        // todo!("CONFIG REPLACE BY STRUCTURES");

        let bind_address = config.bind_address.clone();
        let server = HttpServer::new(move || {
            let jwt_secret = config.jwt_secret.clone();
            let policy = config.permission_policy.clone();
            App::new()
                .app_data(web::Data::new(config.db_pool.clone()))
                .app_data(web::Data::new(config.clone()))
                .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
                .service(
                    web::scope("")
                        .configure(move |cfg| config::api_config(cfg, jwt_secret, policy)),
                )
        })
        .bind(bind_address)?
        .run();
        Ok(Self { server })
    }

    pub async fn run_untill_stopped(self) -> Result<(), std::io::Error> {
        info!("Running application");
        self.server.await
    }
}

pub struct GrpcServer {
    server: Router,
}

impl GrpcServer {
    pub async fn build(config: &Config) -> Result<Self, anyhow::Error> {
        info!("Building gRPC Server");
        let user_service = UserService {
            db_pool: config.db_pool.clone(),
            create_order_crone: config.create_order_crone,
            jwt_secret: config.jwt_secret.clone(),
        };

        let server = Server::builder().add_service(UsersServer::new(user_service));

        Ok(Self { server })
    }

    pub async fn run_untill_stopped(self, config: Config) -> Result<(), tonic::transport::Error> {
        info!("Running gRPC Server");
        self.server
            .serve(
                config
                    .grpc_users_address
                    .parse()
                    .expect("Cannot parse Socket Address"),
            )
            .await
    }
}
