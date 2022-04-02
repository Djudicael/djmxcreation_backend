use serde::{Serialize, Deserialize};


#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    id: Option<i32>,
    bucket_name: String,
    file_name: String,
    mime_type: Option<String>,
}

impl Content {
    pub fn new(id: Option<i32>, bucket_name: String, file_name: String, mime_type: Option<String>) -> Self {
        Self {
            id,
            bucket_name,
            file_name,
            mime_type,
        }
    }

    pub fn id(&self) -> Option<&i32> {
        self.id.as_ref()
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn mime_type(&self) -> Option<&String> {
        self.mime_type.as_ref()
    }
}
