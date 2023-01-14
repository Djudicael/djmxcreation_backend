use std::sync::Arc;

use app_core::{
    about_me::{about_me_repository::DynIAboutMeRepository, about_me_service::DynIAboutMeService},
    project::{project_repository::DynIProjectRepository, project_service::DynIProjectService},
    storage::storage_repository::DynIStorageRepository,
};

use app_service::{about_me_service::AboutMeService, project_service::ProjectService};

use repository::{
    about_me_repository::AboutMeRepository,
    config::{db::Db, minio::StorageClient},
    project_repository::ProjectRepository,
    storage_repository::StorageRepository,
};

#[derive(Clone)]
pub struct ServiceRegister {
    pub project_service: DynIProjectService,
    pub about_me_service: DynIAboutMeService,
}

impl ServiceRegister {
    pub fn new(db: Db, client: StorageClient) -> Self {
        let about_me_db = db.clone();
        let project_repository: DynIProjectRepository = Arc::new(ProjectRepository::new(db));
        let about_me_repository: DynIAboutMeRepository =
            Arc::new(AboutMeRepository::new(about_me_db));
        // let client = get_aws_client("us-west-0").expect("Failed to create AWS client");
        let storage_repository: DynIStorageRepository = Arc::new(StorageRepository::new(client));
        let project_service: DynIProjectService = Arc::new(ProjectService::new(
            project_repository,
            storage_repository.clone(),
        ));

        let about_me_service: DynIAboutMeService =
            Arc::new(AboutMeService::new(about_me_repository, storage_repository));
        Self {
            project_service,
            about_me_service,
        }
    }
}
