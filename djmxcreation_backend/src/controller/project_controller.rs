use crate::{
    app_error::{self, Error, WebErrorMessage},
    domain::{metadata::Metadata, project_content_entity::ProjectContentEntity},
    mapper::project_mapper::to_entity,
    service::project_service::{
        add_project, create_project, delete_project, delete_project_content, find_project,
        get_portfolio_projects, update_project,
    },
    view::{content_view::ContentView, project_payload::ProjectPayload, project_view::ProjectView},
};
use bytes::BufMut;
use futures::{TryFutureExt, TryStreamExt};
use serde_json::json;
use uuid::Uuid;
use warp::{hyper::StatusCode, multipart::FormData, Rejection};

pub async fn handler_create_project(body: Metadata) -> Result<impl warp::Reply, Rejection> {
    let new_project = handle_error(create_project(&body).await)?;
    let tmpjson = json!(new_project);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_add_project(id: i32, form: FormData) -> Result<impl warp::Reply, Rejection> {
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
    let mut contents: Vec<ContentView> = vec![];
    if let Ok(parts) = uploaded {
        for (name, buffer) in parts.into_iter() {
            let content_view = handle_error(add_project(id, name, &buffer).await)?;
            contents.push(content_view);
        }
    };
    let tmpjson = json!(contents);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_update_project(
    id: i32,
    project: ProjectPayload,
) -> Result<impl warp::Reply, Rejection> {
    let project_entity = to_entity(&project);

    handle_error(update_project(id, &project_entity).await)?;

    Ok("done")
}

pub async fn handler_find_project(id: i32) -> Result<impl warp::Reply, Rejection> {
    // let project = find_project(id).await.unwrap();
    let project = handle_error(find_project(id).await)?;
    let tmpjson = json!(project);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_delete_project(id: i32) -> Result<impl warp::Reply, Rejection> {
    handle_error(delete_project(id).await)?;
    Ok("deleted")
}

pub async fn handler_delete_content_project(
    id: i32,
    content_id: i32,
) -> Result<impl warp::Reply, Rejection> {
    handle_error(delete_project_content(id, content_id).await)?;
    Ok("deleted")
}

pub async fn handler_get_projects() -> Result<impl warp::Reply, Rejection> {
    let projects = handle_error(get_portfolio_projects().await)?;

    let tmpjson = json!({ "projects": projects });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

fn handle_error<T>(result: Result<T, Error>) -> Result<T, Rejection> {
    result.map_err(|my_error| match my_error {
        app_error::Error::EntityNotFound(_) => {
            eprintln!("ICI");
            warp::reject::not_found()
        }
        _ => {
            eprintln!("la");
            WebErrorMessage::rejection("Internal Server Error", "".to_string())
        }
    })
}
