use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::metadata::Metadata;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPayload {
    metadata: Option<Metadata>,
    description: Option<Value>,
    visible: bool,
}

impl ProjectPayload {
    pub fn new(metadata: Option<Metadata>, description: Option<Value>, visible: bool) -> Self {
        Self {
            metadata,
            description,
            visible,
        }
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    pub fn description(&self) -> Option<&Value> {
        self.description.as_ref()
    }

    pub fn visible(&self) -> bool {
        self.visible
    }
}
