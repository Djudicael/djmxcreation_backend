use crate::{error::error::ApiResult, service::service_register::ServiceRegister};
use app_core::home::{DynIHomeService, HomeSettings};
use axum::{Extension, Json, Router, routing::get};

pub struct HomeRouter;

impl HomeRouter {
    pub fn new_router(services: ServiceRegister) -> Router {
        Router::new()
            .route("/v1", get(Self::public))
            .route("/v1/settings", get(Self::settings).put(Self::save))
            .layer(Extension(services.home_service))
    }

    async fn public(
        Extension(service): Extension<DynIHomeService>,
    ) -> ApiResult<Json<HomeSettings>> {
        Ok(Json(service.get(true).await?))
    }

    async fn settings(
        Extension(service): Extension<DynIHomeService>,
    ) -> ApiResult<Json<HomeSettings>> {
        Ok(Json(service.get(false).await?))
    }

    async fn save(
        Extension(service): Extension<DynIHomeService>,
        Json(settings): Json<HomeSettings>,
    ) -> ApiResult<Json<HomeSettings>> {
        Ok(Json(service.save(settings).await?))
    }
}
