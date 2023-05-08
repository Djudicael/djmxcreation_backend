use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::view::spotlight_view::SpotlightView;

pub type DynISpotlightService = Arc<dyn ISpotlightService + Send + Sync>;

#[async_trait]
pub trait ISpotlightService {
    async fn add_spotlight(&self, project_id: i32) -> Result<SpotlightView, Error>;
    async fn get_spotlight(&self, id: i32) -> Result<SpotlightView, Error>;
    async fn get_spotlights(&self) -> Result<Vec<SpotlightView>, Error>;
    async fn delete_spotlight(&self, id: i32) -> Result<(), Error>;
}
