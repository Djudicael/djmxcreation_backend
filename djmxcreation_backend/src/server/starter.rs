use std::convert::Infallible;

use serde_json::json;
use warp::{Rejection, Reply};

use crate::{app_error::Error, router::about_me::about_me_filter};

pub async fn start() -> Result<(), Error> {
    // Apis
    let apis = about_me_filter();
    // let routes = apis.recover(handle_rejection);
    let routes = apis;
    println!("start web server");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

    Ok(())
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    // Print to server side
    println!("ERROR - {:?}", err);

    // TODO - Call log API for capture and store

    // Build user message
    // let user_message = match err.find::<WebErrorMessage>() {
    // 	Some(err) => err.typ.to_string(),
    // 	None => "Unknown".to_string(),
    // };

    let result = json!({ "errorMessage": "user_message" });
    let result = warp::reply::json(&result);

    Ok(warp::reply::with_status(
        result,
        warp::http::StatusCode::BAD_REQUEST,
    ))
}
