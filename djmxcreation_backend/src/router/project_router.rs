use warp::Filter;

use crate::controller::project_controller::{
    handler_add_project, handler_create_project, handler_delete_content_project,
    handler_delete_project, handler_find_project, handler_update_project,
};

pub fn project_filter() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let project_base = warp::path("project");

    let create_project = project_base
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(handler_create_project);

    let add_project = project_base
        .and(warp::patch())
        .and(warp::path::param())
        .and(warp::multipart::form().max_length(5_000_000))
        .and(warp::path("content"))
        .and(warp::path::end())
        .and_then(handler_add_project);

    let find_project = project_base
        .and(warp::get())
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(handler_find_project);

    let update_project = project_base
        .and(warp::put())
        .and(warp::path::param())
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(handler_update_project);

    let delete_project = project_base
        .and(warp::delete())
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(handler_delete_project);

    let delete_content_project = project_base
        .and(warp::delete())
        .and(warp::path::param())
        .and(warp::path("content"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(handler_delete_content_project);

    create_project
        .or(update_project)
        .or(add_project)
        .or(find_project)
        .or(delete_project)
        .or(delete_content_project)
}
