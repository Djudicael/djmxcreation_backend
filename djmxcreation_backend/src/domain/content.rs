use serde::{Serialize, Deserialize};


#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    id: i64,
    bucket_name: String,
    file_name: String,
    mime_type: String,
}

impl Content {
    pub fn new(id: i64, bucket_name: String, file_name: String, mime_type: String) -> Self {
        Self {
            id,
            bucket_name,
            file_name,
            mime_type,
        }
    }

    pub fn id(&self) -> &i64 {
        &self.id
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }
}
