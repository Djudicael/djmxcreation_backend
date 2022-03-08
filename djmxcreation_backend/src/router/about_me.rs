use std::convert::Infallible;

use serde::Serialize;
use serde_json::json;
use warp::{hyper::StatusCode, reply::Json, Filter};

use crate::domain::about_me::AboutMe;

pub fn about_me_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let about_me_base = warp::path("me");

    // LIST todos `GET me/`
    let get_me = about_me_base
        .and(warp::get())
        .and(warp::path::end())
        .and_then(handler_get_about_me);

    let update_me = about_me_base
        .and(warp::patch())
        .and(warp::path::param())
        .and(warp::body::json())
        .and_then(handler_update_about_me);
    get_me.or(update_me)
}

pub async fn handler_get_about_me() -> Result<impl warp::Reply, Infallible> {
    let about = AboutMe::new(
        "Judicael".to_string(),
        "dubray".to_string(),
        Some("tata".to_string()),
        None,
    );
    // let about = AboutMe::new(
    //     "Judicael".to_string(),
    //     "dubray".to_string(),
    //     Some("tata".to_string()),
    //     Some("yoyo".to_string()),
    // );
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

    println!("idi");
    let tmpjson = json!({"id":id, "data": body });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

fn json_response<D: Serialize>(data: D) -> Result<Json, warp::Rejection> {
    let response = json!(data);
    Ok(warp::reply::json(&response))
}
