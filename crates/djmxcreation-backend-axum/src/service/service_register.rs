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
    config::{db::ClientV2, minio::StorageClient},
    contact_repository::ContactRepository,
    project_repository::ProjectRepository,
    spotlight_repository::SpotlightRepository,
    storage_repository::StorageRepository,
};
use tokio::sync::Mutex;

#[derive(Clone)]

pub struct ServiceRegister {
    pub project_service: DynIProjectService,
    pub about_me_service: DynIAboutMeService,
    pub contact_service: DynIContactService,
}

impl ServiceRegister {
    pub fn new(db_v2: ClientV2, client: StorageClient) -> Self {
        let client_db = Arc::new(Mutex::new(db_v2));
        let project_repository = Arc::new(ProjectRepository::new(client_db.clone()));
        let about_me_repository = Arc::new(AboutMeRepository::new(client_db.clone()));
        let contact_repository = Arc::new(ContactRepository::new(client_db.clone()));
        let spotlight_repository = Arc::new(SpotlightRepository::new(client_db.clone()));
        let storage_repository = Arc::new(StorageRepository::new(client.clone()));

        Self {
            project_service: Arc::new(ProjectService::new(
                project_repository.clone(),
                storage_repository.clone(),
                spotlight_repository.clone(),
            )),
            about_me_service: Arc::new(AboutMeService::new(
                about_me_repository.clone(),
                storage_repository.clone(),
            )),
            contact_service: Arc::new(ContactService::new(contact_repository.clone())),
        }
    }
}
