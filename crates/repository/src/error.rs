use app_error::Error;

pub fn to_error(sqlx_error: sqlx::Error, message: Option<String>) -> Error {
    println!("sqlx_error: {sqlx_error:?}");
    match sqlx_error {
        sqlx::Error::RowNotFound => Error::EntityNotFound(message.unwrap_or("".to_string())),
        _ => Error::Database, // TODO print the stack trace
    }
}
