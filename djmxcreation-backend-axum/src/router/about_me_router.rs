use crate::{
    controller::about_me_controller::{
        handle_add_image_profile_to_about_me, handler_delete_image_about_me, handler_get_about_me,
        handler_update_about_me,
    },
    error::axum_error::ApiResult,
};
use app_domain::view::about_me_view::AboutMeView;
use axum::{
    extract::{Multipart, Path},
    routing::{delete, get, post, put},
    Json, Router,
};
pub fn about_me_router() -> Router {
    Router::new()
        .route("/v1/me", get(handler_get_about_me))
        .route("/v1/me/:id", put(update_about_me))
        .route("/v1/me/:id", delete(delete_image_about_me))
        .route("/v1/me/:id/image", post(add_image_profile_to_about_me))
}

pub async fn update_about_me(
    Path(id): Path<i32>,
    Json(body): Json<AboutMeView>,
) -> ApiResult<Json<AboutMeView>> {
    handler_update_about_me(id, body).await
}
pub async fn delete_image_about_me(Path(id): Path<i32>) -> ApiResult<()> {
    handler_delete_image_about_me(id).await
}
pub async fn add_image_profile_to_about_me(Path(id): Path<i32>, form: Multipart) -> ApiResult<()> {
    handle_add_image_profile_to_about_me(id, form).await
}
