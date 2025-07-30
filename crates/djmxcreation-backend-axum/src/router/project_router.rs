use app_core::{
    dto::{metadata_dto::MetadataDto, project_dto::ProjectDto},
    project::project_service::DynIProjectService,
    view::{
        content_view::ContentView, project_payload::ProjectPayload, project_view::ProjectView,
        projects_view::ProjectsView, spotlight_view::SpotlightView,
    },
};

use axum::{
    Extension, Json, Router,
    extract::{Multipart, Path, Query},
    routing::{delete, get, patch, post, put},
};
use serde::{Deserialize, Serialize};

use tower_http::limit::{RequestBodyLimit, RequestBodyLimitLayer};
use uuid::Uuid;

use crate::{error::axum_error::ApiResult, service::service_register::ServiceRegister};

pub struct ProjectRouter;

#[derive(Deserialize)]
pub struct Params {
    id: Uuid,
    content_id: Uuid,
}

#[derive(Deserialize)]
pub struct PaginationQueryParams {
    pub page: i64,
    pub size: i64,
    pub adult: Option<bool>,
    pub visible: bool,
}

impl Default for PaginationQueryParams {
    fn default() -> Self {
        Self {
            page: 0,
            size: 10,
            adult: None,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectToAdd {
    project_id: Uuid,
}

impl ProjectRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route(
                "/v1/projects",
                post(Self::create_project).get(Self::get_projects),
            )
            .route("/v2/projects", get(Self::get_projects_with_filter))
            .route("/v1/projects/{id}/contents", patch(Self::add_project))
            .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10 MB
            .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024)) // 50 MB
            .route("/v1/projects/{id}", put(Self::update_project))
            .route(
                "/v1/projects/{id}/thumbnails/{content_id}",
                put(Self::add_thumbnail_to_project),
            )
            .route(
                "/v1/projects/{id}",
                get(Self::find_project).delete(Self::delete_project),
            )
            .route(
                "/v1/projects/{id}/contents/{content_id}",
                delete(Self::delete_content_project),
            )
            .route(
                "/v1/projects/spotlights",
                post(Self::add_spotlight).get(Self::get_spotlights),
            )
            .route(
                "/v1/projects/spotlights/{id}",
                get(Self::get_spotlight).delete(Self::delete_spotlight),
            )
            .layer(Extension(service_register.project_service))
    }

    pub async fn get_projects(
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<Json<Vec<ProjectView>>> {
        let projects = project_service.get_portfolio_projects().await?;
        Ok(Json(projects))
    }

    pub async fn get_projects_with_filter(
        Extension(project_service): Extension<DynIProjectService>,
        Query(pagination): Query<PaginationQueryParams>,
    ) -> ApiResult<Json<ProjectsView>> {
        let projects = project_service
            .get_projects_with_filter(
                pagination.page,
                pagination.size,
                pagination.adult,
                pagination.visible,
            )
            .await?;

        Ok(Json(projects))
    }

    pub async fn create_project(
        Extension(project_service): Extension<DynIProjectService>,
        Json(body): Json<MetadataDto>,
    ) -> ApiResult<Json<ProjectView>> {
        let new_project = project_service.create_project(&body).await?;
        Ok(Json(new_project))
    }

    pub async fn add_project(
        Extension(project_service): Extension<DynIProjectService>,
        Path(id): Path<Uuid>,
        mut form: Multipart,
    ) -> ApiResult<Json<Vec<ContentView>>> {
        let mut contents: Vec<ContentView> = vec![];

        while let Some(field) = form.next_field().await? {
            let uudi_v4 = Uuid::new_v4().to_string();
            println!("[multipart] field.name: {:?}", field.name());
            println!("[multipart] field.file_name: {:?}", field.file_name());
            println!("[multipart] field.content_type: {:?}", field.content_type());
            let file_name = if let Some(file_name) = field.file_name() {
                format!("{}-{}", uudi_v4, file_name.to_owned())
            } else {
                uudi_v4
            };
            println!("File name: {}", file_name);
            let bytes = match field.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Error reading multipart field bytes: {:?}", e);
                    return Err(e.into());
                }
            };
            println!("[multipart] bytes.len: {}", bytes.len());
            let content_view = project_service.add_project(id, file_name, &bytes).await?;

            contents.push(content_view);
        }

        Ok(Json(contents))
    }

    pub async fn update_project(
        Extension(project_service): Extension<DynIProjectService>,
        Path(id): Path<Uuid>,
        Json(project): Json<ProjectPayload>,
    ) -> ApiResult<()> {
        project_service
            .update_project(id, &ProjectDto::from(project))
            .await?;
        Ok(())
    }

    pub async fn find_project(
        Path(id): Path<Uuid>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<Json<ProjectView>> {
        let project = project_service.find_project(id).await?;
        Ok(Json(project))
    }

    pub async fn delete_project(
        Path(id): Path<Uuid>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<()> {
        project_service.delete_project(id).await?;
        Ok(())
    }

    // alternative to the struct:
    //  Path((id, content_id)): Path<(i32, i32)
    pub async fn delete_content_project(
        Path(Params { id, content_id }): Path<Params>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<()> {
        project_service
            .delete_project_content(id, content_id)
            .await?;
        Ok(())
    }
    pub async fn add_thumbnail_to_project(
        Path(Params { id, content_id }): Path<Params>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<Json<ContentView>> {
        let thumbnail = project_service
            .add_thumbnail_to_project(id, content_id)
            .await?;
        Ok(Json(thumbnail))
    }

    pub async fn get_spotlights(
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<Json<Vec<SpotlightView>>> {
        let spotlights = project_service.get_spotlights().await?;
        Ok(Json(spotlights))
    }

    pub async fn get_spotlight(
        Extension(project_service): Extension<DynIProjectService>,
        Path(spotlight_id): Path<Uuid>,
    ) -> ApiResult<Json<SpotlightView>> {
        let spotlight = project_service.get_spotlight(spotlight_id).await?;
        Ok(Json(spotlight))
    }

    pub async fn add_spotlight(
        Extension(project_service): Extension<DynIProjectService>,
        Json(payload): Json<ProjectToAdd>,
    ) -> ApiResult<Json<SpotlightView>> {
        let spotlight = project_service.add_spotlight(payload.project_id).await?;
        Ok(Json(spotlight))
    }

    pub async fn delete_spotlight(
        Extension(project_service): Extension<DynIProjectService>,
        Path(spotlight_id): Path<Uuid>,
    ) -> ApiResult<()> {
        project_service.delete_spotlight(spotlight_id).await?;
        Ok(())
    }
}
