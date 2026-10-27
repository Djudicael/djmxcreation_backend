use app_error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HomeSettings {
    pub introduction: String,
    pub featured_project_id: Option<Uuid>,
}

pub type DynIHomeRepository = Arc<dyn IHomeRepository + Send + Sync>;
pub type DynIHomeService = Arc<dyn IHomeService + Send + Sync>;

#[async_trait]
pub trait IHomeRepository {
    async fn get(&self, public: bool) -> Result<HomeSettings, Error>;
    async fn save(&self, settings: &HomeSettings) -> Result<HomeSettings, Error>;
}

#[async_trait]
pub trait IHomeService {
    async fn get(&self, public: bool) -> Result<HomeSettings, Error>;
    async fn save(&self, settings: HomeSettings) -> Result<HomeSettings, Error>;
}
