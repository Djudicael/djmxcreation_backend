use serde_json::Value;
use sqlx::types::{chrono, Json};

use super::metadata::Metadata;

pub struct ProjectEntity {
    id: Option<i32>,
    metadata: Metadata,
    description: Option<Json<Value>>,
    visible: bool,
    created_on: chrono::DateTime<chrono::Utc>,
    updated_on: Option<chrono::DateTime<chrono::Utc>>,
}
