use std::sync::{Arc, Mutex};

use app_core::{
    dto::{
        content_dto::ContentDto, project_content_dto::ProjectContentDto, project_dto::ProjectDto,
    },
    project::project_repository::{DynIProjectRepository, IProjectRepository},
};
use app_error::Error;
use async_trait::async_trait;
use uuid::Uuid;

struct State {
    project_by_id: Option<ProjectDto>,
    project_content_for_add: Option<ProjectContentDto>,
    project_content_by_id: Option<ProjectContentDto>,
    thumbnail_by_id: Option<ProjectContentDto>,
    added_content_requests: Mutex<Vec<(Uuid, ContentDto)>>,
    deleted_content_requests: Mutex<Vec<(Uuid, Uuid)>>,
    deleted_thumbnail_requests: Mutex<Vec<(Uuid, Uuid)>>,
}

struct FakeProjectRepository {
    state: Arc<State>,
}

pub struct ProjectRepositoryProbe {
    state: Arc<State>,
}

impl ProjectRepositoryProbe {
    pub fn added_content_requests(&self) -> Vec<(Uuid, ContentDto)> {
        self.state
            .added_content_requests
            .lock()
            .expect("project add lock poisoned")
            .clone()
    }

    pub fn deleted_content_requests(&self) -> Vec<(Uuid, Uuid)> {
        self.state
            .deleted_content_requests
            .lock()
            .expect("project delete content lock poisoned")
            .clone()
    }

    pub fn deleted_thumbnail_requests(&self) -> Vec<(Uuid, Uuid)> {
        self.state
            .deleted_thumbnail_requests
            .lock()
            .expect("project delete thumbnail lock poisoned")
            .clone()
    }
}

#[async_trait]
impl IProjectRepository for FakeProjectRepository {
    async fn create(
        &self,
        _metadata: &app_core::dto::metadata_dto::MetadataDto,
    ) -> Result<ProjectDto, Error> {
        unreachable!("create is not used in these service tests")
    }

    async fn add_project_content(
        &self,
        project_id: Uuid,
        content: &ContentDto,
    ) -> Result<ProjectContentDto, Error> {
        self.state
            .added_content_requests
            .lock()
            .expect("project add lock poisoned")
            .push((project_id, content.clone()));
        Ok(self
            .state
            .project_content_for_add
            .clone()
            .expect("missing project_content_for_add fixture"))
    }

    async fn add_project_thumbnail(
        &self,
        _project_id: Uuid,
        _thumbnail: &ContentDto,
    ) -> Result<ProjectContentDto, Error> {
        unreachable!("add_project_thumbnail is not used in these service tests")
    }

    async fn get_project_by_id(&self, _id: Uuid) -> Result<Option<ProjectDto>, Error> {
        Ok(self.state.project_by_id.clone())
    }

    async fn get_projects(&self) -> Result<Vec<ProjectDto>, Error> {
        unreachable!("get_projects is not used in these service tests")
    }

    async fn get_projects_with_filter(
        &self,
        _page: i64,
        _size: i64,
        _is_adult: Option<bool>,
        _is_visible: bool,
    ) -> Result<app_core::dto::projects_dto::ProjectsDto, Error> {
        unreachable!("get_projects_with_filter is not used in these service tests")
    }

    async fn update_project_entity(
        &self,
        _project_id: Uuid,
        _project: &ProjectDto,
    ) -> Result<(), Error> {
        unreachable!("update_project_entity is not used in these service tests")
    }

    async fn get_projects_contents(
        &self,
        _project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        unreachable!("get_projects_contents is not used in these service tests")
    }

    async fn get_projects_content_by_id(
        &self,
        _project_id: Uuid,
        _id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error> {
        Ok(self.state.project_content_by_id.clone())
    }

    async fn delete_project_content_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.state
            .deleted_content_requests
            .lock()
            .expect("project delete content lock poisoned")
            .push((project_id, id));
        Ok(())
    }

    async fn delete_project_by_id(&self, _project_id: Uuid) -> Result<(), Error> {
        unreachable!("delete_project_by_id is not used in these service tests")
    }

    async fn get_projects_content_thumbnail(
        &self,
        _project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error> {
        unreachable!("get_projects_content_thumbnail is not used in these service tests")
    }

    async fn delete_thumbnail_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.state
            .deleted_thumbnail_requests
            .lock()
            .expect("project delete thumbnail lock poisoned")
            .push((project_id, id));
        Ok(())
    }

    async fn get_thumbnail_by_id(
        &self,
        _project_id: Uuid,
        _id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error> {
        Ok(self.state.thumbnail_by_id.clone())
    }
}

pub fn create_project_repository(
    project_by_id: Option<ProjectDto>,
    project_content_for_add: Option<ProjectContentDto>,
    project_content_by_id: Option<ProjectContentDto>,
    thumbnail_by_id: Option<ProjectContentDto>,
) -> (DynIProjectRepository, ProjectRepositoryProbe) {
    let state = Arc::new(State {
        project_by_id,
        project_content_for_add,
        project_content_by_id,
        thumbnail_by_id,
        added_content_requests: Mutex::new(Vec::new()),
        deleted_content_requests: Mutex::new(Vec::new()),
        deleted_thumbnail_requests: Mutex::new(Vec::new()),
    });

    let repository: DynIProjectRepository = Arc::new(FakeProjectRepository {
        state: state.clone(),
    });
    let probe = ProjectRepositoryProbe { state };

    (repository, probe)
}
