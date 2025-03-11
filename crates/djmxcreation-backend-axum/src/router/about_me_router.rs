use crate::{error::axum_error::ApiResult, service::service_register::ServiceRegister};
use app_core::{
    about_me::about_me_service::DynIAboutMeService,
    dto::about_me_dto::AboutMeDto,
    view::{about_me_view::AboutMeView, me_view::MeView},
};

use axum::{
    Extension, Json, Router,
    extract::{Multipart, Path},
    routing::{get, post, put},
};
use uuid::Uuid;

pub struct AboutMeRouter;

impl AboutMeRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/v1/me", get(AboutMeRouter::get_about_me))
            .route(
                "/v1/me/{id}",
                put(AboutMeRouter::update_about_me).delete(AboutMeRouter::delete_image_about_me),
            )
            .route(
                "/v1/me/{id}/image",
                post(AboutMeRouter::add_image_profile_to_about_me),
            )
            .layer(Extension(service_register.about_me_service))
    }

    pub async fn get_about_me(
        Extension(about_me_service): Extension<DynIAboutMeService>,
    ) -> ApiResult<Json<MeView>> {
        let about_me = about_me_service.about_me().await?;
        Ok(Json(about_me))
    }

    pub async fn update_about_me(
        Extension(about_me_service): Extension<DynIAboutMeService>,
        Path(id): Path<Uuid>,
        Json(body): Json<AboutMeView>,
    ) -> ApiResult<Json<MeView>> {
        let about_me = about_me_service
            .update_me(id, &AboutMeDto::from(body))
            .await?;
        Ok(Json(about_me))
    }
    pub async fn delete_image_about_me(
        Extension(about_me_service): Extension<DynIAboutMeService>,
        Path(id): Path<Uuid>,
    ) -> ApiResult<()> {
        about_me_service.delete_photo(id).await?;
        Ok(())
    }

    pub async fn add_image_profile_to_about_me(
        Extension(about_me_service): Extension<DynIAboutMeService>,
        Path(id): Path<Uuid>,
        mut form: Multipart,
    ) -> ApiResult<()> {
        while let Some(field) = form.next_field().await? {
            let uudi_v4 = Uuid::new_v4().to_string();
            let file_name = if let Some(file_name) = field.file_name() {
                format!("{}-{}", uudi_v4, file_name.to_owned())
            } else {
                uudi_v4
            };

            about_me_service
                .add_profile_picture(id, file_name, &field.bytes().await?)
                .await?;
        }
        Ok(())
    }
}
