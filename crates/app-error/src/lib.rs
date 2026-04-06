use thiserror::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error")]
    Database,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Failed to upload file to storage")]
    StorageUpload,
    #[error("Failed to get object URL from storage")]
    StorageGetObjectUrl,
    #[error("Failed to create bucket")]
    BucketCreation,
    #[error("Failed to set public ACL for bucket")]
    PublicBucketAcl,
    #[error("Failed to delete object from storage")]
    StorageDeleteObject,
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Content not found: {0}")]
    ContentNotFoundButWasSave(String),
}
