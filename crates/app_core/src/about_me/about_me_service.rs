use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub type DynIAboutMeService = Arc<dyn IAboutMeService + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type DynIAboutMeService = Arc<dyn IAboutMeService + Sync>;
use uuid::Uuid;

use crate::{dto::about_me_dto::AboutMeDto, view::me_view::MeView};

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait IAboutMeService {
    async fn about_me(&self) -> Result<MeView, Error>;
    async fn update_me(&self, id: Uuid, about: &AboutMeDto) -> Result<MeView, Error>;
    async fn add_profile_picture(
        &self,
        id: Uuid,
        file_name: String,
        file: &[u8],
    ) -> Result<(), Error>;
    async fn delete_photo(&self, id: Uuid) -> Result<(), Error>;
}
