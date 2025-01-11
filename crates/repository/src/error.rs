use app_error::Error;
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
