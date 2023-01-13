use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::{dto::about_me_dto::AboutMeDto, view::me_view::MeView};

pub type DynIAboutMeService = Arc<dyn IAboutMeService + Send + Sync>;

#[async_trait]
pub trait IAboutMeService {
    async fn about_me(&self) -> Result<MeView, Error>;
    async fn update_me(&self, id: i32, about: &AboutMeDto) -> Result<MeView, Error>;
    async fn add_profile_picture(
        &self,
        id: i32,
        file_name: String,
        file: &[u8],
    ) -> Result<(), Error>;
    async fn delete_photo(&self, id: i32) -> Result<(), Error>;
}
