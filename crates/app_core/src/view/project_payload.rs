use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::{metadata_dto::MetadataDto, project_dto::ProjectDto};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPayload {
    pub metadata: Option<MetadataDto>,
    pub description: Option<Value>,
    pub visible: bool,
    pub adult: bool,
}

impl ProjectPayload {
    pub fn new(
        metadata: Option<MetadataDto>,
        description: Option<Value>,
        visible: bool,
        adult: bool,
    ) -> Self {
        Self {
            metadata,
            description,
            visible,
            adult,
        }
    }
}

impl From<ProjectPayload> for ProjectDto {
    fn from(val: ProjectPayload) -> Self {
        ProjectDto::new()
            .metadata(val.metadata)
            .description(val.description)
            .visible(val.visible)
            .adult(val.adult)
    }
}
