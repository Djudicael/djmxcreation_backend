use serde::Serialize;
use serde_json::json;
use warp::{reply::Json, Filter};

use crate::controller::about_me_controller::{
    handler_create_about_me, handler_delete_image_about_me, handler_get_about_me,
    handler_update_about_me,
};

pub fn about_me_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let about_me_base = warp::path("me");

    // `GET me/`
    let get_me = about_me_base
        .and(warp::get())
        .and(warp::path::end())
        .and_then(handler_get_about_me);

    let create_me = about_me_base
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(handler_create_about_me);

    let update_me = about_me_base
        .and(warp::patch())
        .and(warp::path::param())
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(handler_update_about_me);

    let delete_image_me = about_me_base
        .and(warp::delete())
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(handler_delete_image_about_me);

    get_me.or(create_me).or(update_me).or(delete_image_me)
}

fn json_response<D: Serialize>(data: D) -> Result<Json, warp::Rejection> {
    let response = json!(data);
    Ok(warp::reply::json(&response))
}
