use app_error::Error;

use deadpool_postgres::PoolError;
use serde_json::Error as SerdeJsonError;

use tokio_postgres::{error::SqlState, Error as PgError};

pub fn to_error(error: PoolError, message: Option<String>) -> Error {
    println!("Error: {:?}", error);

    match error {
        PoolError::Backend(pg_error) => handle_pg_error(&pg_error, message),
        PoolError::Timeout(_) => {
            println!("Pool timeout error");
            Error::Database
        }
        PoolError::NoRuntimeSpecified => {
            println!("Pool configuration error: No runtime specified");
            Error::Database
        }
        _ => {
            println!("Unexpected pool error: {:?}", error);
            Error::Database
        }
    }
}

fn handle_pg_error(error: &PgError, message: Option<String>) -> Error {
    if let Some(db_error) = error.as_db_error() {
        println!("Database error code: {:?}", db_error.code());

        match db_error.code() {
            // Table/view not found
            code if code == &SqlState::UNDEFINED_TABLE => {
                Error::EntityNotFound(message.unwrap_or_else(|| "Entity not found".to_string()))
            }
            // Invalid parameter value
            code if code == &SqlState::INVALID_PARAMETER_VALUE => Error::InvalidInput(
                message.unwrap_or_else(|| "Invalid parameter value".to_string()),
            ),
            // No data found
            code if code == &SqlState::NO_DATA_FOUND => {
                Error::EntityNotFound(message.unwrap_or_else(|| "Entity not found".to_string()))
            }
            // Foreign key violation
            code if code == &SqlState::FOREIGN_KEY_VIOLATION => Error::InvalidInput(
                message.unwrap_or_else(|| "Referenced entity not found".to_string()),
            ),
            // Unique violation
            code if code == &SqlState::UNIQUE_VIOLATION => {
                Error::InvalidInput(message.unwrap_or_else(|| "Entity already exists".to_string()))
            }
            // Not null violation
            code if code == &SqlState::NOT_NULL_VIOLATION => Error::InvalidInput(
                message.unwrap_or_else(|| "Required field is missing".to_string()),
            ),
            // Other database errors
            _ => {
                println!("Unhandled database error: {:?}", error);
                Error::Database
            }
        }
    } else {
        println!("Non-database error: {:?}", error);
        Error::Database
    }
}

pub fn handle_serde_json_error(error: SerdeJsonError) -> Error {
    println!("Serde JSON error: {error}");
    Error::Database // Return a general database error, but can be customized further
}

pub fn handle_uuid_error(error: uuid::Error) -> Error {
    println!(" uuid error: {error}");
    Error::Database // Return a general database error, but can be customized further
}
