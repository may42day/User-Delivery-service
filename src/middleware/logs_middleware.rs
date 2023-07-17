use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use tracing::{info, Span};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{filter, EnvFilter, Layer};

pub struct CustomRootSpanBuilder;

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        tracing_actix_web::root_span!(request, error = tracing::field::Empty,)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

pub async fn init_tracing_suscriber() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(tracing::Level::DEBUG.as_str()));
    let request_logs_file_appender = tracing_appender::rolling::hourly("/var/log/", "requests_logs.log");
    let debug_logs_file_appender = tracing_appender::rolling::hourly("/var/log/", "debug_logs.log");

    let requests_log =
        BunyanFormattingLayer::new("delivery_user".into(), request_logs_file_appender);
    let debug_log = tracing_subscriber::fmt::layer();
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(requests_log)
        .with(stdout_log)
        .with(
            debug_log
                .with_writer(debug_logs_file_appender)
                .with_ansi(false)
                .with_filter(filter::LevelFilter::DEBUG),
        );

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");
    info!("Tracing subscriber inited")
}
