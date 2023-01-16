use aws_sdk_s3::{error::PutObjectError, types::SdkError};
use s3::error::S3Error;
use thiserror::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Web server failed to start because web-folder '{0}' not found.")]
    FailStartWebFolderNotFound(String),
    #[error("Fail authentication missing X-Auth-Token header.")]
    FailAuthMissingXAuth,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    // #[error(transparent)]
    // SqlxNotRowFoundError(#[from] sqlx::Error::RowNotFound),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    StorageUpload(#[from] SdkError<PutObjectError>),
    #[error("Error bucket creation")]
    BucketCreation,
    #[error(transparent)]
    StorageDeleteObject(#[from] aws_sdk_s3::types::SdkError<aws_sdk_s3::error::DeleteObjectError>),
    #[error(transparent)]
    S3(#[from] S3Error),
    #[error("Entity Not Found - {0}] ")]
    EntityNotFound(String),
}

#[derive(Debug)]
pub struct WebErrorMessage {
    pub typ: &'static str,
    pub message: String,
}

impl warp::reject::Reject for WebErrorMessage {}

impl WebErrorMessage {
    pub fn rejection(typ: &'static str, message: String) -> warp::Rejection {
        warp::reject::custom(WebErrorMessage { typ, message })
    }
}

impl From<self::Error> for warp::Rejection {
    fn from(other: self::Error) -> Self {
        WebErrorMessage::rejection("web::Error", format!("{other}"))
    }
}
// impl From<model::Error> for warp::Rejection {
//     fn from(other: model::Error) -> Self {
//         WebErrorMessage::rejection("model::Error", format!("{}", other))
//     }
// }
// impl From<security::Error> for warp::Rejection {
//     fn from(other: security::Error) -> Self {
//         WebErrorMessage::rejection("security::Error", format!("{}", other))
//     }
// }
