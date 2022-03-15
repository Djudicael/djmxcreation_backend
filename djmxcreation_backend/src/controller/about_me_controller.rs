use std::convert::Infallible;

use serde_json::json;
use warp::hyper::StatusCode;

use crate::domain::about_me::AboutMe;

pub async fn handler_get_about_me() -> Result<impl warp::Reply, Infallible> {
    let about = AboutMe::new(
        Some(1),
        "Judicael".to_string(),
        "dubray".to_string(),
        Some("tata".to_string()),
        None,
    );

    let tmpjson = json!(about);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_update_about_me(
    id: i64,
    body: AboutMe,
) -> Result<impl warp::Reply, Infallible> {
    let tmpjson = json!({"id":id, "data": body });
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
