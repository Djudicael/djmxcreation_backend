use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::{about_me_dto::AboutMeDto, content_dto::ContentDto};

pub type DynIAboutMeRepository = Arc<dyn IAboutMeRepository + Send + Sync>;

#[async_trait]
pub trait IAboutMeRepository {
    async fn update_about_me(&self, id: Uuid, about: &AboutMeDto) -> Result<AboutMeDto, Error>;
    async fn get_about_me(&self) -> Result<AboutMeDto, Error>;
    async fn get_about_me_by_id(&self, id: Uuid) -> Result<AboutMeDto, Error>;
    async fn update_photo(&self, id: Uuid, content: &ContentDto) -> Result<(), Error>;
    async fn delete_about_me_photo(&self, id: Uuid) -> Result<(), Error>;
}
