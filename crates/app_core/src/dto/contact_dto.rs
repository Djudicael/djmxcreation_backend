use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ContactDto {
    pub id: Option<Uuid>,
    pub description: Option<Value>,
}

impl ContactDto {
    pub fn new(id: Option<Uuid>, description: Option<Value>) -> Self {
        Self { id, description }
    }
}
