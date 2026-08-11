use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),

    #[error(transparent)]
    Kafka(#[from] rdkafka::error::KafkaError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Multipart(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) | AppError::Io(_) | AppError::Kafka(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, "request failed");
        }

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
