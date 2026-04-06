use crate::error::axum_error::Error;

/// Sanitise a user-supplied file name received from a multipart upload.
///
/// Rejects names that contain path traversal sequences or other dangerous
/// characters that could be used to escape the intended storage prefix.
pub fn validate_file_name(name: &str) -> Result<&str, Error> {
    if name.is_empty() {
        return Err(Error::BadRequest("file name must not be empty".to_string()));
    }

    if name.contains("..")
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
    {
        return Err(Error::BadRequest(
            "file name contains invalid characters".to_string(),
        ));
    }

    Ok(name)
}

/// Maximum page size allowed for paginated queries.
const MAX_PAGE_SIZE: i64 = 100;

/// Validate pagination parameters from query strings.
pub fn validate_pagination(page: i64, size: i64) -> Result<(), Error> {
    if page < 1 {
        return Err(Error::BadRequest(
            "page must be greater than 0".to_string(),
        ));
    }
    if size < 1 || size > MAX_PAGE_SIZE {
        return Err(Error::BadRequest(
            format!("size must be between 1 and {MAX_PAGE_SIZE}"),
        ));
    }
    Ok(())
}
