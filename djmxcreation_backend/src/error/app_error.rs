use thiserror::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Web server failed to start because web-folder '{0}' not found.")]
    FailStartWebFolderNotFound(String),

    #[error("Fail authentication missing X-Auth-Token header.")]
    FailAuthMissingXAuth,
}
