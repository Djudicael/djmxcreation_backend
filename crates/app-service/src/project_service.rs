use app_core::{
    dto::{
        content_dto::ContentDto, metadata_dto::MetadataDto, project_content_dto::ProjectContentDto,
        project_dto::ProjectDto, projects_dto::ProjectsDto, spotlight_dto::SpotlightDto,
    },
    mapper::project_mapper::to_view,
    project::{project_repository::DynIProjectRepository, project_service::IProjectService},
    spotlight::spotlight_repository::DynISpotlightRepository,
    storage::storage_repository::DynIStorageRepository,
    view::{
        content_view::ContentView, project_view::ProjectView,
        project_with_thumbnail_view::ProjectWithThumbnailView, projects_view::ProjectsView,
        spotlight_view::SpotlightView,
    },
};
use app_error::Error;
use async_trait::async_trait;
use futures::future::join_all;
use tracing::{debug, warn};
use uuid::Uuid;

pub struct ProjectService {
    pub project_repository: DynIProjectRepository,
    pub storage_repository: DynIStorageRepository,
    pub spotlight_repository: DynISpotlightRepository,
    /// Name of the S3 bucket where all media is stored.
    bucket: String,
}

impl ProjectService {
    pub fn new(
        project_repository: DynIProjectRepository,
        storage_repository: DynIStorageRepository,
        spotlight_repository: DynISpotlightRepository,
        bucket: String,
    ) -> Self {
        Self {
            project_repository,
            storage_repository,
            spotlight_repository,
            bucket,
        }
    }

    async fn resolve_url(&self, file_name: &str) -> Option<String> {
        match self
            .storage_repository
            .get_object_url(&self.bucket, file_name)
            .await
        {
            Ok(url) => Some(url),
            Err(e) => {
                warn!(bucket = %self.bucket, file = %file_name, error = ?e, "failed to resolve storage URL");
                None
            }
        }
    }

    async fn to_spotlight_view(&self, spotlight: &SpotlightDto) -> Result<SpotlightView, Error> {
        let thumbnail = match &spotlight.thumbnail {
            Some(photo) => {
                let url = self.resolve_url(&photo.file_name).await;
                Some(ContentView::new(photo.id, photo.mime_type.clone(), url))
            }
            None => None,
        };

        Ok(SpotlightView::new(
            spotlight.id,
            spotlight.project_id,
            spotlight.adult,
            spotlight.metadata.clone(),
            spotlight.created_on,
            thumbnail,
        ))
    }

    async fn to_contents(&self, project_contents: &[ProjectContentDto]) -> Vec<ContentView> {
        let futures: Vec<_> = project_contents
            .iter()
            .filter_map(|dto| {
                dto.content.as_ref().map(|photo| {
                    let id = dto.id;
                    let mime_type = photo.mime_type.clone();
                    let file_name = photo.file_name.clone();
                    async move {
                        let url = self.resolve_url(&file_name).await;
                        ContentView::new(id, mime_type, url)
                    }
                })
            })
            .collect();

        join_all(futures).await
    }
}

#[async_trait]
impl IProjectService for ProjectService {
    async fn create_project(&self, metadata: &MetadataDto) -> Result<ProjectView, Error> {
        debug!(title = ?metadata.title, "creating project");
        let project = self.project_repository.create(metadata).await?;
        Ok(to_view(&[], &project))
    }

    async fn add_project(
        &self,
        id: Uuid,
        file_name: String,
        file: &[u8],
    ) -> Result<ContentView, Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;
        let key = format!("{}/{}", self.bucket, file_name);
        let content = ContentDto::new(None, self.bucket.clone(), key.clone(), None);
        self.storage_repository
            .upload_file(&self.bucket, &key, file)
            .await?;
        let content_dto = self
            .project_repository
            .add_project_content(id, &content)
            .await?;
        let (url, mime_type) = match content_dto.content {
            Some(ref photo) => (
                self.resolve_url(&photo.file_name).await,
                photo.mime_type.clone(),
            ),
            None => (None, None),
        };
        Ok(ContentView::new(content_dto.id, mime_type, url))
    }

    async fn add_thumbnail_to_project(
        &self,
        id: Uuid,
        content_id: Uuid,
    ) -> Result<ContentView, Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;
        let project_content = self
            .project_repository
            .get_projects_content_by_id(id, content_id)
            .await?
            .ok_or_else(|| {
                Error::EntityNotFound(format!("content {content_id} not found in project {id}"))
            })?;

        match project_content.content {
            Some(photo) => {
                let mut content = photo.clone();
                content.id = project_content.id;
                let thumbnail_saved = self
                    .project_repository
                    .add_project_thumbnail(id, &content)
                    .await?;
                let url = self
                    .storage_repository
                    .get_object_url(&self.bucket, &photo.file_name)
                    .await?;
                Ok(ContentView::new(
                    thumbnail_saved.id,
                    photo.mime_type,
                    Some(url),
                ))
            }
            None => Err(Error::ContentNotFoundButWasSave(
                "content record exists but has no associated file".to_string(),
            )),
        }
    }

    async fn update_project(&self, id: Uuid, project: &ProjectDto) -> Result<(), Error> {
        self.project_repository
            .get_project_by_id(id)
            .await?
            .ok_or_else(|| Error::EntityNotFound(format!("project {id} not found")))?;
        self.project_repository
            .update_project_entity(id, project)
            .await
    }

    async fn find_project(&self, id: Uuid) -> Result<ProjectView, Error> {
        let project_entity = self
            .project_repository
            .get_project_by_id(id)
            .await?
            .ok_or_else(|| Error::EntityNotFound(format!("project {id} not found")))?;

        let mut project_view = ProjectView::from(project_entity.clone());
        let contents = self.to_contents(&project_entity.contents).await;

        let thumbnail = match project_entity.thumbnail {
            Some(ref photo) => {
                let url = self.resolve_url(&photo.file_name).await;
                Some(ContentView::new(photo.id, photo.mime_type.clone(), url))
            }
            None => None,
        };

        project_view.contents = contents;
        project_view.thumbnail = thumbnail;
        Ok(project_view)
    }

    async fn delete_project(&self, id: Uuid) -> Result<(), Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;
        let project_contents = self.project_repository.get_projects_contents(id).await?;
        self.project_repository.delete_project_by_id(id).await?;

        for content_dto in project_contents {
            if let Some(content) = content_dto.content {
                if let Err(e) = self
                    .storage_repository
                    .remove_object(&self.bucket, &content.file_name)
                    .await
                {
                    // Log but don't fail — DB row is already deleted.
                    warn!(file = %content.file_name, error = ?e, "failed to delete orphaned storage object");
                }
            }
        }

        Ok(())
    }

    async fn delete_project_content(
        &self,
        project_id: Uuid,
        content_id: Uuid,
    ) -> Result<(), Error> {
        self.project_repository
            .get_project_by_id(project_id)
            .await?
            .ok_or_else(|| Error::EntityNotFound(format!("project {project_id} not found")))?;

        let content_dto = self
            .project_repository
            .get_projects_content_by_id(project_id, content_id)
            .await?
            .ok_or_else(|| Error::EntityNotFound(format!("content {content_id} not found")))?;

        self.project_repository
            .delete_project_content_by_id(project_id, content_id)
            .await?;

        if let Some(content) = content_dto.content {
            self.storage_repository
                .remove_object(&self.bucket, &content.file_name)
                .await?;
        }

        // If this content was the project thumbnail, remove that entry too.
        match self
            .project_repository
            .get_thumbnail_by_id(project_id, content_id)
            .await
        {
            Ok(Some(thumbnail)) => {
                if let Some(thumb_id) = thumbnail.id {
                    self.project_repository
                        .delete_thumbnail_by_id(project_id, thumb_id)
                        .await?;
                }
            }
            Ok(None) => {}
            Err(e) => {
                warn!(content_id = %content_id, error = ?e, "error checking thumbnail during content deletion");
            }
        }

        Ok(())
    }

    async fn get_portfolio_projects(&self) -> Result<Vec<ProjectView>, Error> {
        let projects = self.project_repository.get_projects().await?;

        let futures: Vec<_> = projects
            .into_iter()
            .map(|data| async move {
                let contents = self.to_contents(&data.contents).await;

                let thumbnail_view = match &data.thumbnail {
                    Some(photo) => {
                        let url = self.resolve_url(&photo.file_name).await;
                        Some(ContentView::new(photo.id, photo.mime_type.clone(), url))
                    }
                    None => None,
                };

                ProjectView::new()
                    .id(data.id)
                    .description(data.description)
                    .metadata(data.metadata)
                    .visible(data.visible)
                    .adult(data.adult)
                    .created_on(data.created_on)
                    .updated_on(data.updated_on)
                    .thumbnail(thumbnail_view)
                    .contents(contents)
                    .build()
            })
            .collect();

        Ok(join_all(futures).await)
    }

    async fn get_projects_with_filter(
        &self,
        page: i64,
        size: i64,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsView, Error> {
        let ProjectsDto {
            total_pages,
            projects,
            ..
        } = self
            .project_repository
            .get_projects_with_filter(page, size, is_adult, is_visible)
            .await?;

        let futures: Vec<_> = projects
            .into_iter()
            .map(|data| async move {
                let thumbnail_view = match &data.thumbnail {
                    Some(photo) => {
                        let url = self.resolve_url(&photo.file_name).await;
                        Some(ContentView::new(photo.id, photo.mime_type.clone(), url))
                    }
                    None => None,
                };

                ProjectWithThumbnailView::new(
                    data.id,
                    data.metadata,
                    data.visible,
                    data.adult,
                    data.created_on,
                    data.updated_on,
                    thumbnail_view,
                )
            })
            .collect();

        Ok(ProjectsView::new(page, size, total_pages, join_all(futures).await))
    }

    async fn add_spotlight(&self, project_id: Uuid) -> Result<SpotlightView, Error> {
        self.project_repository
            .get_project_by_id(project_id)
            .await?
            .ok_or_else(|| Error::EntityNotFound(format!("project {project_id} not found")))?;
        let spotlight = self.spotlight_repository.add_spotlight(project_id).await?;
        self.to_spotlight_view(&spotlight).await
    }

    async fn get_spotlight(&self, spotlight_id: Uuid) -> Result<SpotlightView, Error> {
        let spotlight = self
            .spotlight_repository
            .get_spotlight(spotlight_id)
            .await?
            .ok_or_else(|| Error::EntityNotFound(format!("spotlight {spotlight_id} not found")))?;
        self.to_spotlight_view(&spotlight).await
    }

    async fn get_spotlights(&self) -> Result<Vec<SpotlightView>, Error> {
        let spotlights = self.spotlight_repository.get_spotlights().await?;

        let futures: Vec<_> = spotlights
            .iter()
            .map(|data| self.to_spotlight_view(data))
            .collect();

        let results = join_all(futures).await;
        Ok(results
            .into_iter()
            .filter_map(|r| r.inspect_err(|e| warn!(error = ?e, "failed to build spotlight view")).ok())
            .collect())
    }

    async fn delete_spotlight(&self, spotlight_id: Uuid) -> Result<(), Error> {
        self.spotlight_repository
            .delete_spotlight(spotlight_id)
            .await
    }
}
