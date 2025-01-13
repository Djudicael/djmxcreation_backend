use app_error::Error;
use serde_json::Error as SerdeJsonError;
use tokio_postgres::{error::SqlState, Error as PgError};

pub fn to_error(pg_error: PgError, message: Option<String>) -> Error {
    println!("pg_error: {pg_error:?}");
    if let Some(db_error) = pg_error.as_db_error() {
        if db_error.code() == &SqlState::UNDEFINED_TABLE {
            Error::EntityNotFound(message.unwrap_or_else(|| "Entity not found".to_string()))
        } else {
            // Print detailed error information for debugging purposes
            if let Some(cause) = pg_error.code() {
                println!("Underlying cause: {cause:?}");
            }
            Error::Database
        }
    } else {
        Error::Database
    }
}

pub fn handle_serde_json_error(error: SerdeJsonError) -> Error {
    println!("Serde JSON error: {error}");
    Error::Database // Return a general database error, but can be customized further
}
