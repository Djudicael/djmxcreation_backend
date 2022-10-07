use app_domain::about_me::AboutMe;
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

pub async fn add_image_profile_to_about_me(id: i32, mut form: Multipart) -> ApiResult<()> {
    while let Some(mut field) = form.next_field().await.unwrap() {
    //    let test =  field. .bytes().await.unwrap(); 
    //         .and_then(|part| {
    //             // let name = part.name().to_string();
    //             let file_name = part.filename().unwrap_or_default();
    //             let uudi_v4 = Uuid::new_v4().to_string();

    //             let name = if file_name.is_empty() {
    //                 uudi_v4
    //             } else {
    //                 uudi_v4 + "-" + file_name
    //             };
    //             let value = part.stream().try_fold(Vec::new(), |mut vec, data| {
    //                 vec.put(data);
    //                 async move { Ok(vec) }
    //             });
    //             value.map_ok(move |vec| (name, vec))
    //         })
    //         .try_collect()
    //         .await
    //         .map_err(|e| {
    //             panic!("multipart error: {:?}", e);
    //         });
    }
    // if let Ok(parts) = uploaded {
    //     for (name, buffer) in parts.into_iter() {
    //         add_profile_picture(id, name, &buffer).await.unwrap();
    //     }
    // };
    Ok(())
}
