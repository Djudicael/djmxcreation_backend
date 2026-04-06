use app_error::Error;
use deadpool_postgres::PoolError;
use serde_json::Error as SerdeJsonError;
use tokio_postgres::{error::SqlState, Error as PgError};
use tracing::{error, warn};

pub fn to_error(pool_error: PoolError, message: Option<String>) -> Error {
    match pool_error {
        PoolError::Backend(pg_error) => handle_pg_error(&pg_error, message),
        PoolError::Timeout(_) => {
            error!("database pool timeout");
            Error::Database
        }
        PoolError::NoRuntimeSpecified => {
            error!("database pool misconfigured: no async runtime specified");
            Error::Database
        }
        _ => {
            error!(error = ?pool_error, "unexpected pool error");
            Error::Database
        }
    }
}

fn handle_pg_error(error: &PgError, message: Option<String>) -> Error {
    if let Some(db_error) = error.as_db_error() {
        match db_error.code() {
            code if code == &SqlState::UNDEFINED_TABLE => {
                Error::EntityNotFound(message.unwrap_or_else(|| "entity not found".to_string()))
            }
            code if code == &SqlState::INVALID_PARAMETER_VALUE => {
                Error::InvalidInput(message.unwrap_or_else(|| "invalid parameter value".to_string()))
            }
            code if code == &SqlState::NO_DATA_FOUND => {
                Error::EntityNotFound(message.unwrap_or_else(|| "entity not found".to_string()))
            }
            code if code == &SqlState::FOREIGN_KEY_VIOLATION => {
                Error::InvalidInput(message.unwrap_or_else(|| "referenced entity not found".to_string()))
            }
            code if code == &SqlState::UNIQUE_VIOLATION => {
                Error::InvalidInput(message.unwrap_or_else(|| "entity already exists".to_string()))
            }
            code if code == &SqlState::NOT_NULL_VIOLATION => {
                Error::InvalidInput(message.unwrap_or_else(|| "required field is missing".to_string()))
            }
            _ => {
                error!(error = ?error, "unhandled postgres error");
                Error::Database
            }
        }
    } else {
        warn!(error = ?error, "non-database postgres error");
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
