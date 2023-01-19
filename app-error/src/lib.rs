use aws_sdk_s3::{error::PutObjectError, types::SdkError};
use s3::error::S3Error;
use thiserror::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Web server failed to start because web-folder '{0}' not found.")]
    FailStartWebFolderNotFound(String),
    #[error("Fail authentication missing X-Auth-Token header.")]
    FailAuthMissingXAuth,
    #[error("Database error")]
    Database,
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
