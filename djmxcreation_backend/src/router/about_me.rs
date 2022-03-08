mod domain;
pub fn about_me_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let about_me_base = warp::path("me");

    // LIST todos `GET me/`
	let get_me = about_me_base
    .and(warp::get())
    .and(warp::path::end())
    .and_then(handler_get_aboutme);
    get_me
}

pub async fn handler_get_aboutme(
    
) -> Result<impl warp::Reply, Infallible> {
    let about = AboutMe::new("Judicael", "dubray", Some("tata"), Some("yoyo"));
    let tmpjson = json!({ "aboutme": about });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}


fn json_response<D: Serialize>(data: D) -> Result<Json, warp::Rejection> {
	let response = json!(data);
	Ok(warp::reply::json(&response))
}
