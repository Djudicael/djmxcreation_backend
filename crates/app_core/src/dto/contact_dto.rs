use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ContactDto {
    pub id: Option<i32>,
    pub description: Option<Value>,
}

impl ContactDto {
    pub fn new(id: Option<i32>, description: Option<Value>) -> Self {
        Self { id, description }
    }
}
