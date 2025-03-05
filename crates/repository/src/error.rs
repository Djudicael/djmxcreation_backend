use app_error::Error;
use serde_json::Error as SerdeJsonError;
use tokio_postgres::{error::SqlState, Error as PgError};

pub fn to_error(pg_error: PgError, message: Option<String>) -> Error {
    println!("pg_error: {pg_error:?}");

    match pg_error.to_string().as_str() {
        s if s.contains("RowCount") => {
            return Error::EntityNotFound(
                message.unwrap_or_else(|| "Entity not found".to_string()),
            );
        }
        _ => {}
    }

    if let Some(db_error) = pg_error.as_db_error() {
        println!("db_error code: {:?}", db_error.code());
        match db_error.code() {
            code if code == &SqlState::UNDEFINED_TABLE => {
                Error::EntityNotFound(message.unwrap_or_else(|| "Entity not found".to_string()))
            }
            code if code == &SqlState::INVALID_PARAMETER_VALUE => Error::InvalidInput(
                message.unwrap_or_else(|| "Invalid parameter value".to_string()),
            ),
            code if code == &SqlState::NO_DATA_FOUND => {
                Error::EntityNotFound(message.unwrap_or_else(|| "Entity not found".to_string()))
            }
            _ => {
                // Print detailed error information for debugging purposes
                println!("Database error code: {:?}", db_error.code());
                Error::Database
            }
        }
    } else {
        // Handle non-database errors
        println!("Non-database error: {:?}", pg_error);
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
