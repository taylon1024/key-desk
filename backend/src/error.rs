use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::db::DbError;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound,
    Conflict(String),
    Internal(&'static str),
}

impl From<DbError> for ApiError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::NotFound => ApiError::NotFound,
            DbError::Conflict => ApiError::Conflict("同一作用域下变量名已存在".to_string()),
            DbError::Other(err) => {
                tracing::error!(error = %err, "database error");
                ApiError::Internal("数据库错误")
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "变量不存在".to_string()),
            ApiError::Conflict(message) => (StatusCode::CONFLICT, message),
            ApiError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.to_string()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
