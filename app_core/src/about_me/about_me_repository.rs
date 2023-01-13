use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::dto::{about_me_dto::AboutMeDto, content_dto::ContentDto};

pub type DynIAboutMeRepository = Arc<dyn IAboutMeRepository + Send + Sync>;

#[async_trait]
pub trait IAboutMeRepository {
    async fn update_about_me(&self, id: i32, about: &AboutMeDto) -> Result<AboutMeDto, Error>;
    async fn get_about_me(&self) -> Result<AboutMeDto, Error>;
    async fn get_about_me_by_id(&self, id: i32) -> Result<AboutMeDto, Error>;
    async fn update_photo(&self, id: i32, content: &ContentDto) -> Result<(), Error>;
    async fn delete_about_me_photo(&self, id: i32) -> Result<(), Error>;
}
