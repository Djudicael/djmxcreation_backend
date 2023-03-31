use app_core::{contact::contact_service::DynIContactService, dto::contact_dto::ContactDto};
use axum::{
    extract::Path,
    routing::{get, put},
    Extension, Json, Router,
};

use crate::{error::axum_error::ApiResult, service::service_register::ServiceRegister};

pub struct ContactRouter;

impl ContactRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/v1/information", get(ContactRouter::get_contact))
            .route("/v1/information/:id", put(ContactRouter::update_contact))
            .layer(Extension(service_register.contact_service))
    }

    pub async fn get_contact(
        Extension(contact_service): Extension<DynIContactService>,
    ) -> ApiResult<Json<ContactDto>> {
        let contact = contact_service.get_contact().await?;
        Ok(Json(contact))
    }

    pub async fn update_contact(
        Extension(contact_service): Extension<DynIContactService>,
        Path(id): Path<i32>,
        Json(body): Json<ContactDto>,
    ) -> ApiResult<Json<ContactDto>> {
        let contact = contact_service.update_contact(id, &body).await?;
        Ok(Json(contact))
    }
}
