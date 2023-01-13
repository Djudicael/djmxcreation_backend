use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::dto::{metadata_dto::MetadataDto, project_dto::ProjectDto};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPayload {
    pub metadata: Option<MetadataDto>,
    pub description: Option<Value>,
    pub visible: bool,
}

impl ProjectPayload {
    pub fn new(metadata: Option<MetadataDto>, description: Option<Value>, visible: bool) -> Self {
        Self {
            metadata,
            description,
            visible,
        }
    }
}

impl Into<ProjectDto> for ProjectPayload {
    fn into(self) -> ProjectDto {
        let metadata_json = self.metadata.map(|meta| json!(meta));
        let description_json = self.description.map(|description| json!(description));

        ProjectDto::new(
            None,
            metadata_json,
            description_json,
            self.visible,
            None,
            None,
        )
    }
}
