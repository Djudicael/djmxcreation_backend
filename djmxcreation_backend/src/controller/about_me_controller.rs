use std::convert::Infallible;

use serde_json::json;
use sqlx::types::Json;
use warp::hyper::StatusCode;

use crate::domain::about_me::AboutMe;
use crate::mapper::about_me_mapper::*;
use crate::service::about_me_service::*;
use crate::view::about_me_view::AboutMeView;

pub async fn handler_get_about_me() -> Result<impl warp::Reply, Infallible> {
    let about = about_me().await.unwrap();
    let view = to_view(
        about.id().map(|id| *id),
        about.first_name(),
        about.last_name(),
        about.description().map(|description| &description.0),
        about.photo().map(|photo| photo.clone()),
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
) -> Result<impl warp::Reply, Infallible> {
    let about = to_model(&body);
    let new_me = update_me(id, &about).await.unwrap();
    let view = to_view(
        new_me.id().map(|id| *id),
        new_me.first_name(),
        new_me.last_name(),
        new_me.description().map(|description| &description.0),
        new_me.photo().map(|photo| photo.clone()),
    );
    let tmpjson = json!(view);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_delete_image_about_me(id: i64) -> Result<impl warp::Reply, Infallible> {
    let tmpjson = json!({ "id": id });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_create_about_me(body: AboutMe) -> Result<impl warp::Reply, Infallible> {
    let tmpjson = json!({ "data": body });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}
