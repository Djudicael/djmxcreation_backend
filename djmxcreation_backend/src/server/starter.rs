use std::convert::Infallible;

use serde_json::json;
use warp::hyper::StatusCode;
use warp::Filter;
use warp::{Rejection, Reply};

use crate::{app_error::Error, router::about_me::about_me_filter};

pub async fn start() -> Result<(), Error> {
    // Apis
    let apis = about_me_filter();
    // let routes = apis.recover(handle_rejection);
    let routes = apis.recover(handle_rejection);
    println!("start web server");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

    Ok(())
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
