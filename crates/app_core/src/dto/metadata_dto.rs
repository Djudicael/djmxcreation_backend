use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataDto {
    pub title: Option<String>,
    pub sub_title: Option<String>,
    pub client: Option<String>,
}

impl MetadataDto {
    pub fn new(title: Option<String>, sub_title: Option<String>, client: Option<String>) -> Self {
        Self {
            title,
            sub_title,
            client,
        }
    }
}
