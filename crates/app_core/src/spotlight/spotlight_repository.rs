use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::dto::spotlight_dto::SpotlightDto;

pub type DynISpotlightRepository = Arc<dyn ISpotlightRepository + Send + Sync>;

#[async_trait]
pub trait ISpotlightRepository {
    async fn add_spotlight(&self, project_id: i32) -> Result<SpotlightDto, Error>;
    async fn get_spotlights(&self) -> Result<Vec<SpotlightDto>, Error>;
    async fn get_spotlight(&self, id: i32) -> Result<SpotlightDto, Error>;
    async fn delete_spotlight(&self, id: i32) -> Result<(), Error>;
}
