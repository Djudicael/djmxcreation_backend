use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub type DynISpotlightRepository = Arc<dyn ISpotlightRepository + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type DynISpotlightRepository = Arc<dyn ISpotlightRepository + Sync>;
use uuid::Uuid;

use crate::dto::spotlight_dto::SpotlightDto;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ISpotlightRepository {
    async fn add_spotlight(&self, project_id: Uuid) -> Result<SpotlightDto, Error>;
    async fn get_spotlights(&self) -> Result<Vec<SpotlightDto>, Error>;
    async fn get_spotlight(&self, id: Uuid) -> Result<Option<SpotlightDto>, Error>;
    async fn delete_spotlight(&self, id: Uuid) -> Result<(), Error>;
}
