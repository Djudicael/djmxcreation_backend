use axum::extract::Multipart;

use app_domain::mapper::about_me_mapper::*;
use app_domain::view::about_me_view::AboutMeView;

use app_service::about_me_service::*;

use axum::Json;

use uuid::Uuid;

use crate::error::axum_error::ApiResult;

pub async fn handler_get_about_me() -> ApiResult<Json<AboutMeView>> {
    let about = about_me().await?;
    let view = to_view(
        about.id().cloned(),
        about.first_name(),
        about.last_name(),
        about.description().map(|description| &description.0),
        about.photo_url().cloned(), // about.photo().map(|photo| photo.clone()),
    );

    Ok(Json(view))
}

pub async fn handler_update_about_me(id: i32, body: AboutMeView) -> ApiResult<Json<AboutMeView>> {
    let about = to_model(&body);
    let new_me = update_me(id, &about).await?;
    let view = to_view(
        new_me.id().cloned(),
        new_me.first_name(),
        new_me.last_name(),
        new_me.description().map(|description| &description.0),
        // new_me.photo().map(|photo| photo.clone()),
        None,
    );
    Ok(Json(view))
}

pub async fn handler_delete_image_about_me(id: i32) -> ApiResult<()> {
    delete_photo(id).await?;
    Ok(())
}

pub async fn handle_add_image_profile_to_about_me(id: i32, mut form: Multipart) -> ApiResult<()> {
    while let Some(field) = form.next_field().await.unwrap() {
        let uudi_v4 = Uuid::new_v4().to_string();
        let file_name = if let Some(file_name) = field.file_name() {
            format!("{}-{}", uudi_v4, file_name.to_owned())
        } else {
            uudi_v4
        };

        add_profile_picture(id, file_name, &field.bytes().await?).await?;
    }
    Ok(())
}
