use std::sync::Arc;

use crate::router::{
    about_me_router::AboutMeRouter, contact_router::ContactRouter,
    observability_router::ObservabilityRouter, project_router::ProjectRouter,
};
use crate::service::service_register::ServiceRegister;
use app_config::config::Config;
use axum::Router;
use repository::config::{
    db::DatabaseConfig,
    storage::get_storage_client,
};

pub fn build_router() -> Router {
    let config = Config::from_env().expect("failed to load configuration");
    let client_db = Arc::new(DatabaseConfig::new(&config.database));
    let storage_cfg = config.get_storage();
    let bucket_name = storage_cfg.bucket.clone();

    let storage_client =
        get_storage_client(storage_cfg).expect("failed to create storage client");

    let service_register = ServiceRegister::new(client_db, storage_client, bucket_name);

    Router::new()
        .nest("/", ObservabilityRouter::new_router())
        .nest(
            "/api/about",
            AboutMeRouter::new_router(service_register.clone()),
        )
        .nest(
            "/api/portfolio",
            ProjectRouter::new_router(service_register.clone()),
        )
        .nest(
            "/api/contact",
            ContactRouter::new_router(service_register.clone()),
        )
}
