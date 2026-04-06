use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub errors: HashMap<String, Vec<String>>,
}

impl ApiError {
    pub fn new(message: String) -> Self {
        let mut errors = HashMap::new();
        errors.insert("message".to_owned(), vec![message]);
        Self { errors }
    }
}

pub type ApiResult<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("authentication is required to access this resource")]
    Unauthorized,
    #[error("username or password is incorrect")]
    Forbidden,
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    ApplicationStartup(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("unexpected error has occurred")]
    InternalServerError,
    #[error("service error: {0}")]
    ServiceError(#[from] app_error::Error),
    #[error("multipart upload error: {0}")]
    UploadMultipartError(#[from] MultipartError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Self::ServiceError(ref err) => match err {
                app_error::Error::EntityNotFound(msg)
                | app_error::Error::ContentNotFoundButWasSave(msg) => {
                    (StatusCode::NOT_FOUND, msg.clone())
                }
                app_error::Error::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
                app_error::Error::FailAuthMissingXAuth => {
                    (StatusCode::UNAUTHORIZED, err.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()),
            },
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()),
        };

        (status, Json(ApiError::new(message))).into_response()
    }
}
