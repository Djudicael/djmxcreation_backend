use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDto {
    pub id: Option<Uuid>,
    pub bucket_name: String,
    pub file_name: String,
    pub mime_type: Option<String>,
}

impl ContentDto {
    pub fn new(
        id: Option<Uuid>,
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
