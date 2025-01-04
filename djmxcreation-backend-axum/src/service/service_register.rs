use std::sync::Arc;

use app_core::{
    about_me::about_me_service::DynIAboutMeService, contact::contact_service::DynIContactService,
    project::project_service::DynIProjectService,
    spotlight::spotlight_service::DynISpotlightService,
};

use app_service::{
    about_me_service::AboutMeService, contact_service::ContactService,
    project_service::ProjectService, spotlight_service::SpotlightService,
};

use repository::{
    about_me_repository::AboutMeRepository,
    config::{db::Db, minio::StorageClient},
    contact_repository::ContactRepository,
    project_repository::ProjectRepository,
    spotlight_repository::SpotlightRepository,
    storage_repository::StorageRepository,
};

#[derive(Clone)]

pub struct ServiceRegister {
    pub project_service: DynIProjectService,
    pub about_me_service: DynIAboutMeService,
    pub contact_service: DynIContactService,
    pub spotlight_service: DynISpotlightService,
}

impl ServiceRegister {
    pub fn new(db: Db, client: StorageClient) -> Self {
        let project_repository = Arc::new(ProjectRepository::new(db.clone()));
        let about_me_repository = Arc::new(AboutMeRepository::new(db.clone()));
        let contact_repository = Arc::new(ContactRepository::new(db.clone()));
        let spotlight_repository = Arc::new(SpotlightRepository::new(db.clone()));
        let storage_repository = Arc::new(StorageRepository::new(client.clone()));

        Self {
            project_service: Arc::new(ProjectService::new(
                project_repository.clone(),
                storage_repository.clone(),
            )),
            about_me_service: Arc::new(AboutMeService::new(
                about_me_repository,
                storage_repository.clone(),
            )),
            contact_service: Arc::new(ContactService::new(contact_repository)),
            spotlight_service: Arc::new(SpotlightService::new(
                spotlight_repository,
                project_repository,
                storage_repository,
            )),
        }
    }
}
