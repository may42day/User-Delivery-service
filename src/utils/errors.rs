use std::fmt::{self, Display};
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use tracing::{error, Span};

#[derive(Debug)]
pub struct AppError {
    pub message: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    fn message(&self) -> String {
        match self {
            AppError {
                message: Some(message),
                error_type: _,
            } => message.clone(),
            AppError {
                message: None,
                error_type: _,
            } => "Error description unspecified".to_string(),
        }
    }

    pub fn db_error(error: impl ToString) -> AppError {
        error!("Error with db connection: {:?}", error.to_string());
        Span::current().record("error", error.to_string());
        AppError {
            message: Some("Internal server Error: database error".to_string()),
            error_type: AppErrorType::DbError,
        }
    }

    pub fn grpc_error(error: impl ToString) -> AppError {
        error!("Error with db grpc_connection: {:?}", error.to_string());
        Span::current().record("error", error.to_string());
        AppError {
            message: Some("Internal server Error: gRPC error".to_string()),
            error_type: AppErrorType::GrpcError,
        }
    }

    pub fn serde_error(error: impl ToString) -> AppError {
        error!("Error with serializing object: {:?}", error.to_string());
        Span::current().record("error", error.to_string());
        AppError {
            message: Some("Internal server Error: serializing error".to_string()),
            error_type: AppErrorType::GrpcError,
        }
    }
}

#[derive(Debug)]
pub enum AppErrorType {
    DbError,
    GrpcError,
    SerdeError,
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::DbError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::GrpcError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::SerdeError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}
