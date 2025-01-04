use app_core::{
    spotlight::spotlight_service::DynISpotlightService, view::spotlight_view::SpotlightView,
};
use axum::{
    extract::Path,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{error::axum_error::ApiResult, service::service_register::ServiceRegister};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectToAdd {
    project_id: i32,
}

pub struct SpotlightRouter;

impl SpotlightRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/v1/spotlights", post(Self::add_spotlight))
            .route("/v1/spotlights", get(Self::get_spotlights))
            .route("/v1/spotlights/:id", get(Self::get_spotlight))
            .route("/v1/spotlights/:id", delete(Self::delete_spotlight))
            .layer(Extension(Arc::new(service_register.spotlight_service)))
    }

    pub async fn get_spotlights(
        Extension(spotlight_service): Extension<DynISpotlightService>,
    ) -> ApiResult<Json<Vec<SpotlightView>>> {
        let spotlights = spotlight_service.get_spotlights().await?;
        Ok(Json(spotlights))
    }

    pub async fn get_spotlight(
        Path(id): Path<i32>,
        Extension(spotlight_service): Extension<DynISpotlightService>,
    ) -> ApiResult<Json<SpotlightView>> {
        let spotlight = spotlight_service.get_spotlight(id).await?;
        Ok(Json(spotlight))
    }

    pub async fn add_spotlight(
        Json(body): Json<ProjectToAdd>,
        Extension(spotlight_service): Extension<DynISpotlightService>,
    ) -> ApiResult<Json<SpotlightView>> {
        let spotlight = spotlight_service.add_spotlight(body.project_id).await?;
        Ok(Json(spotlight))
    }

    pub async fn delete_spotlight(
        Path(id): Path<i32>,
        Extension(spotlight_service): Extension<DynISpotlightService>,
    ) -> ApiResult<()> {
        spotlight_service.delete_spotlight(id).await?;
        Ok(())
    }
}
