use bytes::BufMut;
use futures::{TryFutureExt, TryStreamExt};
use serde_json::json;
use uuid::Uuid;
use warp::hyper::StatusCode;
use warp::multipart::FormData;
use warp::Rejection;

use app_domain::about_me::AboutMe;

use app_domain::mapper::about_me_mapper::*;

use app_domain::view::about_me_view::AboutMeView;

use app_service::about_me_service::*;

pub async fn handler_get_about_me() -> Result<impl warp::Reply, Rejection> {
    let about = about_me().await.unwrap();
    let view = to_view(
        about.id().cloned(),
        about.first_name(),
        about.last_name(),
        about.description().map(|description| &description.0),
        about.photo_url().cloned(), // about.photo().map(|photo| photo.clone()),
    );

    let tmpjson = json!(view);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_update_about_me(
    id: i32,
    body: AboutMeView,
) -> Result<impl warp::Reply, Rejection> {
    let about = to_model(&body);
    let new_me = update_me(id, &about).await.unwrap();
    let view = to_view(
        new_me.id().cloned(),
        new_me.first_name(),
        new_me.last_name(),
        new_me.description().map(|description| &description.0),
        // new_me.photo().map(|photo| photo.clone()),
        None,
    );
    let tmpjson = json!(view);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_delete_image_about_me(id: i32) -> Result<impl warp::Reply, Rejection> {
    delete_photo(id).await.unwrap();
    Ok("deleted")
}

//todo  a etudier https://rustcc.cn/article?id=665a3a71-e66a-4029-8a8b-c2db0488ad4b

//async fn upload(ctx: Arc<AppContext>, form: multipart::FormData) -> Result<impl Reply, Infallible> {
pub async fn add_image_profile_to_about_me(
    id: i32,
    form: FormData,
) -> Result<impl warp::Reply, Rejection> {
    let uploaded: Result<Vec<(String, Vec<u8>)>, warp::Rejection> = form
        .and_then(|part| {
            // let name = part.name().to_string();
            let file_name = part.filename().unwrap_or_default();
            let uudi_v4 = Uuid::new_v4().to_string();

            let name = if file_name.is_empty() {
                uudi_v4
            } else {
                uudi_v4 + "-" + file_name
            };
            let value = part.stream().try_fold(Vec::new(), |mut vec, data| {
                vec.put(data);
                async move { Ok(vec) }
            });
            value.map_ok(move |vec| (name, vec))
        })
        .try_collect()
        .await
        .map_err(|e| {
            panic!("multipart error: {:?}", e);
        });
    if let Ok(parts) = uploaded {
        for (name, buffer) in parts.into_iter() {
            add_profile_picture(id, name, &buffer).await.unwrap();
        }
    };
    Ok("done")
}

pub async fn handler_create_about_me(body: AboutMe) -> Result<impl warp::Reply, Rejection> {
    let tmpjson = json!({ "data": body });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}
