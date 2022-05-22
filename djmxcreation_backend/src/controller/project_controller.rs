use crate::{
    domain::metadata::Metadata,
    mapper::project_mapper::to_entity,
    service::project_service::{
        add_project, create_project, delete_project, delete_project_content, find_project,
        update_project,
    },
    view::{content_view::ContentView, project_view::ProjectView},
};
use bytes::BufMut;
use futures::{TryFutureExt, TryStreamExt};
use serde_json::json;
use uuid::Uuid;
use warp::{hyper::StatusCode, multipart::FormData, Rejection};

pub async fn handler_create_project(body: Metadata) -> Result<impl warp::Reply, Rejection> {
    let new_project = create_project(&body).await.unwrap();
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
            let content_view = add_project(id, name, &buffer).await.unwrap();
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
    project: ProjectView,
) -> Result<impl warp::Reply, Rejection> {
    let project_entity = to_entity(&project);

    update_project(id, &project_entity).await.unwrap();

    Ok("done")
}

pub async fn handler_find_project(id: i32) -> Result<impl warp::Reply, Rejection> {
    let project = find_project(id).await.unwrap();
    let tmpjson = json!(project);
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

pub async fn handler_delete_project(id: i32) -> Result<impl warp::Reply, Rejection> {
    delete_project(id).await.unwrap();
    Ok("deleted")
}

pub async fn handler_delete_content_project(
    id: i32,
    content_id: i32,
) -> Result<impl warp::Reply, Rejection> {
    delete_project_content(id, content_id).await.unwrap();
    Ok("deleted")
}
