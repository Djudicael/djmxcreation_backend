use app_domain::{
    mapper::project_mapper::to_entity,
    metadata::Metadata,
    view::{content_view::ContentView, project_payload::ProjectPayload, project_view::ProjectView},
};
use app_service::project_service::{
    add_project, create_project, delete_project, delete_project_content, find_project,
    get_portfolio_projects, update_project,
};
use axum::{extract::Multipart, Json};
use uuid::Uuid;

use crate::error::axum_error::ApiResult;

pub async fn handler_create_project(body: Metadata) -> ApiResult<Json<ProjectView>> {
    let new_project = create_project(&body).await?;
    Ok(Json(new_project))
}

pub async fn handler_add_project(
    id: i32,
    mut form: Multipart,
) -> ApiResult<Json<Vec<ContentView>>> {
    let mut contents: Vec<ContentView> = vec![];

    while let Some(field) = form.next_field().await? {
        let uudi_v4 = Uuid::new_v4().to_string();
        // let content_type = field.get_or_insert("");
        // dbg!(content_type);
        let file_name = if let Some(file_name) = field.file_name() {
            format!("{}-{}", uudi_v4, file_name.to_owned())
        } else {
            uudi_v4
        };

        let content_view = add_project(id, file_name, &field.bytes().await?).await?;
        contents.push(content_view);
    }

    Ok(Json(contents))
}

pub async fn handler_update_project(id: i32, project: ProjectPayload) -> ApiResult<()> {
    update_project(id, &to_entity(&project)).await?;
    Ok(())
}

pub async fn handler_find_project(id: i32) -> ApiResult<Json<ProjectView>> {
    let project = find_project(id).await?;
    Ok(Json(project))
}

pub async fn handler_delete_project(id: i32) -> ApiResult<()> {
    delete_project(id).await?;
    Ok(())
}

pub async fn handler_delete_content_project(id: i32, content_id: i32) -> ApiResult<()> {
    delete_project_content(id, content_id).await?;
    Ok(())
}

pub async fn handler_get_projects() -> ApiResult<Json<Vec<ProjectView>>> {
    let projects = get_portfolio_projects().await?;
    Ok(Json(projects))
}
