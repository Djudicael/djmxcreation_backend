use aws_sdk_s3::{error::PutObjectError, types::SdkError};
use thiserror::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Web server failed to start because web-folder '{0}' not found.")]
    FailStartWebFolderNotFound(String),
    #[error("Fail authentication missing X-Auth-Token header.")]
    FailAuthMissingXAuth,
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    WarpError(#[from] warp::Error),
    #[error(transparent)]
    StorageUploadError(#[from] SdkError<PutObjectError>),
    #[error(transparent)]
    StorageDeleteObjectError(
        #[from] aws_sdk_s3::types::SdkError<aws_sdk_s3::error::DeleteObjectError>,
    ),
}
