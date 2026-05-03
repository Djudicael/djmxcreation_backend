use app_error::Error;
use serde_json::Error as SerdeJsonError;
use tracing::{error, warn};
use wasi_pg_client::PgError;
use wasi_pg_client::error::sqlstate;

/// Convert a `PgError` into our application `Error`.
pub fn to_error(pg_error: PgError, message: Option<String>) -> Error {
    handle_pg_error(pg_error, message)
}

fn handle_pg_error(error: PgError, message: Option<String>) -> Error {
    if let Some(code) = error.code() {
        match code {
            c if c == sqlstate::UNDEFINED_TABLE => {
                Error::EntityNotFound(message.unwrap_or_else(|| "entity not found".to_string()))
            }
            c if c == sqlstate::INVALID_PARAMETER_VALUE => Error::InvalidInput(
                message.unwrap_or_else(|| "invalid parameter value".to_string()),
            ),
            c if c == sqlstate::UNIQUE_VIOLATION => {
                Error::InvalidInput(message.unwrap_or_else(|| "entity already exists".to_string()))
            }
            c if c == sqlstate::FOREIGN_KEY_VIOLATION => Error::InvalidInput(
                message.unwrap_or_else(|| "referenced entity not found".to_string()),
            ),
            c if c == sqlstate::NOT_NULL_VIOLATION => Error::InvalidInput(
                message.unwrap_or_else(|| "required field is missing".to_string()),
            ),
            _ => {
                error!(error = %error, "unhandled postgres error");
                Error::Database
            }
        }
    } else if error.is_connection_broken() {
        error!(error = %error, "database connection broken");
        Error::Database
    } else {
        warn!(error = %error, "non-database postgres error");
        Error::Database
    }
}

pub fn handle_serde_json_error(error: SerdeJsonError) -> Error {
    error!(error = ?error, "serde_json deserialisation error");
    Error::Database
}

pub fn handle_uuid_error(error: uuid::Error) -> Error {
    error!(error = ?error, "uuid parse error");
    Error::Database
}
