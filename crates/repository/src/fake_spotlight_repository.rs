use std::sync::Arc;

use app_core::{
    dto::spotlight_dto::SpotlightDto,
    spotlight::spotlight_repository::{DynISpotlightRepository, ISpotlightRepository},
};
use app_error::Error;
use async_trait::async_trait;
use uuid::Uuid;

struct FakeSpotlightRepository;

#[async_trait]
impl ISpotlightRepository for FakeSpotlightRepository {
    async fn add_spotlight(&self, _project_id: Uuid) -> Result<SpotlightDto, Error> {
        unreachable!("spotlight is not used in these service tests")
    }

    async fn get_spotlights(&self) -> Result<Vec<SpotlightDto>, Error> {
        unreachable!("spotlight is not used in these service tests")
    }

    async fn get_spotlight(&self, _id: Uuid) -> Result<Option<SpotlightDto>, Error> {
        unreachable!("spotlight is not used in these service tests")
    }

    async fn delete_spotlight(&self, _id: Uuid) -> Result<(), Error> {
        unreachable!("spotlight is not used in these service tests")
    }
}

pub fn create_spotlight_repository() -> DynISpotlightRepository {
    Arc::new(FakeSpotlightRepository)
}
