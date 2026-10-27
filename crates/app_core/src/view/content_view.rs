use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::{content_dto::ContentDto, project_content_dto::ProjectContentDto};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentView {
    id: Option<Uuid>,
    mime_type: Option<String>,
    url: Option<String>,
    #[serde(flatten)]
    pub details: crate::media::MediaDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization: Option<crate::optimization::ImageOptimization>,
}

impl ContentView {
    pub fn new(id: Option<Uuid>, mime_type: Option<String>, url: Option<String>) -> Self {
        Self {
            id,
            mime_type,
            url,
            details: Default::default(),
            poster_url: None,
            optimization: None,
        }
    }
}

impl From<ProjectContentDto> for ContentView {
    fn from(dto: ProjectContentDto) -> Self {
        Self::new(dto.id, None, None)
    }
}

impl From<ContentDto> for ContentView {
    fn from(dto: ContentDto) -> Self {
        Self::new(dto.id, dto.mime_type, None)
    }
}
