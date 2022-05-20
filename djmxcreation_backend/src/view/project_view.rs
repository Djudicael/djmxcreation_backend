use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::metadata::Metadata;

use super::content_view::ContentView;
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectView {
    id: Option<i32>,
    metadata: Option<Metadata>,
    description: Option<Value>,
    visible: bool,
    // created_on: Option<chrono::DateTime<chrono::Utc>>,
    // updated_on: Option<chrono::DateTime<chrono::Utc>>,
    contents: Vec<ContentView>,
}

impl ProjectView {
    pub fn new(
        id: Option<i32>,
        metadata: Option<Metadata>,
        description: Option<Value>,
        visible: bool,
        contents: Vec<ContentView>,
    ) -> Self {
        Self {
            id,
            metadata,
            description,
            visible,
            contents,
        }
    }
}
