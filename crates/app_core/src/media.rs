use app_error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub const PART_SIZE: i64 = 8 * 1024 * 1024;
pub const MAX_VIDEO_SIZE: i64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaDetails {
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub alt: String,
    #[serde(default)]
    pub behind_scenes: bool,
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub poster_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UploadRequest {
    pub file_name: String,
    pub mime_type: String,
    pub size: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UploadSession {
    pub id: Uuid,
    pub part_size: i64,
    pub part_count: i32,
}

pub type DynMediaRepository = Arc<dyn IMediaRepository + Send + Sync>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaOrder {
    pub ids: Vec<Uuid>,
    pub previous: Vec<Uuid>,
}

#[async_trait]
pub trait IMediaRepository {
    async fn reorder(&self, project: Uuid, order: MediaOrder) -> Result<(), Error>;
    async fn begin(&self, project: Uuid, request: UploadRequest) -> Result<UploadSession, Error>;
    async fn part_url(&self, project: Uuid, session: Uuid, part: i32) -> Result<String, Error>;
    async fn complete(&self, project: Uuid, session: Uuid) -> Result<Uuid, Error>;
    async fn abort(&self, project: Uuid, session: Uuid) -> Result<(), Error>;
    async fn edit(&self, project: Uuid, content: Uuid, details: MediaDetails) -> Result<(), Error>;
}
