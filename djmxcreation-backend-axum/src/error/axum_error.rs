use std::collections::HashMap;

use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub errors: HashMap<String, Vec<String>>,
}

impl ApiError {
    pub fn new(error: String) -> Self {
        let mut error_map: HashMap<String, Vec<String>> = HashMap::new();
        error_map.insert("message".to_owned(), vec![error]);
        Self { errors: error_map }
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
    #[error("Error from service layer")]
    ServiceError(#[from] app_error::Error),
    #[error("Error from upload file")]
    UploadMultipartError(#[from] MultipartError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::NotFound(err) => (StatusCode::NOT_FOUND, err),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, Self::Unauthorized.to_string()),
            Self::ServiceError(app_error::Error::EntityNotFound(err)) => {
                (StatusCode::NOT_FOUND, err)
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("unexpected error occurred"),
            ),
        };

        //TODO check for better approach
        let body = Json(ApiError::new(error_message));

        (status, body).into_response()
    }
}
