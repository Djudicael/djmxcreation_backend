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
    #[error("Error when upload file to storage")]
    StorageUpload,
    #[error("Error when get object url from storage")]
    StorageGetObjectUrl,
    #[error("Error bucket creation")]
    BucketCreation,
    #[error("Error when delete object from storage")]
    StorageDeleteObject,
    #[error("Entity Not Found - {0}] ")]
    EntityNotFound(String),
}
