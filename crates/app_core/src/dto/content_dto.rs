use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDto {
    pub id: Option<i32>,
    pub bucket_name: String,
    pub file_name: String,
    pub mime_type: Option<String>,
}

impl ContentDto {
    pub fn new(
        id: Option<i32>,
        bucket_name: String,
        file_name: String,
        mime_type: Option<String>,
    ) -> Self {
        Self {
            id,
            bucket_name,
            file_name,
            mime_type,
        }
    }
}
