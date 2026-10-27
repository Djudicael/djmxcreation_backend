use std::sync::Arc;

use app_core::{
    about_me::about_me_service::DynIAboutMeService, contact::contact_service::DynIContactService,
    project::project_service::DynIProjectService,
};
use app_service::{
    about_me_service::AboutMeService, contact_service::ContactService,
    project_service::ProjectService,
};
use repository::{
    about_me_repository::AboutMeRepository,
    config::{db::DatabaseConfig, storage::StorageClient},
    contact_repository::ContactRepository,
    project_repository::ProjectRepository,
    spotlight_repository::SpotlightRepository,
    storage_repository::StorageRepository,
};

#[derive(Clone)]
pub struct ServiceRegister {
    pub optimization: Arc<repository::optimization_repository::OptimizationRepository>,
    pub media_repository: app_core::media::DynMediaRepository,
    pub home_service: app_core::home::DynIHomeService,
    pub project_service: DynIProjectService,
    pub about_me_service: DynIAboutMeService,
    pub contact_service: DynIContactService,
}

impl ServiceRegister {
    pub fn new(client_db: Arc<DatabaseConfig>, client: StorageClient, bucket: String) -> Self {
        let project_repository = Arc::new(ProjectRepository::new(client_db.clone()));
        let about_me_repository = Arc::new(AboutMeRepository::new(client_db.clone()));
        let contact_repository = Arc::new(ContactRepository::new(client_db.clone()));
        let spotlight_repository = Arc::new(SpotlightRepository::new(client_db.clone()));
        let media_repository = Arc::new(repository::media_repository::MediaRepository::new(
            client_db.clone(),
            client.clone(),
            bucket.clone(),
        ));
        let optimization = Arc::new(
            repository::optimization_repository::OptimizationRepository::new(
                client_db.clone(),
                client.clone(),
            ),
        );
        let storage_repository = Arc::new(StorageRepository::new(client));

        Self {
            optimization: optimization.clone(),
            media_repository,
            home_service: Arc::new(app_service::home_service::HomeService::new(Arc::new(
                repository::home_repository::HomeRepository::new(client_db.clone()),
            ))),
            project_service: Arc::new(
                ProjectService::new(
                    project_repository.clone(),
                    storage_repository.clone(),
                    spotlight_repository.clone(),
                    bucket.clone(),
                )
                .with_media_processing(optimization),
            ),
            about_me_service: Arc::new(AboutMeService::new(
                about_me_repository.clone(),
                storage_repository.clone(),
                bucket,
            )),
            contact_service: Arc::new(ContactService::new(contact_repository.clone())),
        }
    }
}
