use app_core::{
    dto::{
        content_dto::ContentDto, metadata_dto::MetadataDto, project_content_dto::ProjectContentDto,
        project_dto::ProjectDto, projects_dto::ProjectsDto,
    },
    mapper::project_mapper::to_view,
    project::{project_repository::DynIProjectRepository, project_service::IProjectService},
    storage::storage_repository::DynIStorageRepository,
    view::{
        content_view::ContentView, project_view::ProjectView,
        project_with_thumbnail_view::ProjectWithThumbnailView, projects_view::ProjectsView,
    },
};
use app_error::Error;
use async_trait::async_trait;
use futures::{stream, FutureExt, StreamExt};

// fn to_content(value: serde_json::Value) -> ContentDto {
//     serde_json::from_value(value).unwrap()
// }

pub struct ProjectService {
    pub project_repository: DynIProjectRepository,
    pub storage_repository: DynIStorageRepository,
}

impl ProjectService {
    pub fn new(
        project_repository: DynIProjectRepository,
        storage_repository: DynIStorageRepository,
    ) -> Self {
        Self {
            project_repository,
            storage_repository,
        }
    }

    async fn to_contents(
        &self,
        project_contents: &Vec<ProjectContentDto>,
    ) -> Result<Vec<ContentView>, Error> {
        let mut contents: Vec<ContentView> = vec![];
        for content_dto in project_contents {
            let content = content_dto.clone().content;
            let (url, mime_type) = match content {
                Some(photo) => {
                    let url = self
                        .storage_repository
                        .get_object_url(&photo.bucket_name, &photo.file_name)
                        .await?;

                    (Some(url), None)
                }
                None => (None, None),
            };
            let content_view = ContentView::new(content_dto.id, mime_type, url);
            contents.push(content_view);
        }
        Ok(contents)
    }
}

#[async_trait]
impl IProjectService for ProjectService {
    async fn create_project(&self, metadata: &MetadataDto) -> Result<ProjectView, Error> {
        let project = self.project_repository.create(metadata).await?;
        let contents: Vec<ContentView> = vec![];
        let project_view = to_view(&contents, &project);
        Ok(project_view)
    }

    async fn add_project(
        &self,
        id: i32,
        file_name: String,
        file: &[u8],
    ) -> Result<ContentView, Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;
        let key = format!("{}/{}", "portfolio", file_name);
        let bucket = "portfolio";
        let content = ContentDto::new(None, bucket.to_owned(), key.clone(), None);
        self.storage_repository
            .upload_file(bucket, key.as_str(), file)
            .await?;
        let content_dto = self
            .project_repository
            .add_project_content(id, &content)
            .await?;
        let content = content_dto.content;
        let (url, mime_type) = match content {
            Some(photo) => {
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await?;

                (Some(url), None)
            }
            None => (None, None),
        };

        let content_view = ContentView::new(content_dto.id, mime_type, url);

        Ok(content_view)
    }

    async fn update_project(&self, id: i32, project: &ProjectDto) -> Result<(), Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;

        self.project_repository
            .update_project_entity(id, project)
            .await?;

        Ok(())
    }

    async fn find_project(&self, id: i32) -> Result<ProjectView, Error> {
        let project_entity = self.project_repository.get_project_by_id(id).await?;

        let project_contents = self.project_repository.get_projects_contents(id).await?;

        let mut contents: Vec<ContentView> = vec![];

        for content_dto in project_contents {
            let content = content_dto.content;
            let (url, mime_type) = match content {
                Some(photo) => {
                    let url = self
                        .storage_repository
                        .get_object_url(&photo.bucket_name, &photo.file_name)
                        .await?;

                    (Some(url), None)
                }
                None => (None, None),
            };

            let content_view = ContentView::new(content_dto.id, mime_type, url);
            contents.push(content_view);
        }
        let project_view = to_view(&contents, &project_entity);
        Ok(project_view)
    }

    async fn delete_project(&self, id: i32) -> Result<(), Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;
        let project_contents = self.project_repository.get_projects_contents(id).await?;
        self.project_repository.delete_project_by_id(id).await?;
        for content_dto in project_contents {
            let content = content_dto.content;

            if let Some(content) = content {
                self.storage_repository
                    .remove_object(&content.bucket_name, &content.file_name)
                    .await?
            }
        }

        Ok(())
    }

    async fn delete_project_content(&self, project_id: i32, content_id: i32) -> Result<(), Error> {
        let _ = self
            .project_repository
            .get_project_by_id(project_id)
            .await?;
        let content_dto = self
            .project_repository
            .get_projects_content_by_id(project_id, content_id)
            .await?;
        let content = content_dto.content;

        self.project_repository
            .delete_project_content_by_id(project_id, content_id)
            .await?;

        if let Some(content) = content {
            self.storage_repository
                .remove_object(&content.bucket_name, &content.file_name)
                .await?
        }

        Ok(())
    }

    async fn get_portfolio_projects(&self) -> Result<Vec<ProjectView>, Error> {
        let projects = self.project_repository.get_projects().await?;

        let result = stream::iter(projects)
            .fold(Vec::new(), |mut vec, data| async move {
                let contents = match data.id {
                    Some(id) => {
                        let thumb = self
                            .project_repository
                            .get_projects_content_thumbnail(id)
                            .await
                            .unwrap();
                        self.to_contents(&thumb).await.unwrap()
                    }
                    None => vec![],
                };
                let project_view = to_view(&contents, &data);
                vec.push(project_view);
                vec
            })
            .map(move |vec| vec)
            .await;

        Ok(result)
    }

    async fn get_projects_with_filter(
        &self,
        page: i32,
        size: i32,
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
        let result = stream::iter(projects)
            .fold(Vec::new(), |mut vec, data| async move {
                let content = data.thumbnail;
                let (url, mime_type, id) = match content {
                    Some(photo) => {
                        let id = photo.id;
                        let url = self
                            .storage_repository
                            .get_object_url(&photo.bucket_name, &photo.file_name)
                            .await
                            .unwrap(); // TODO handle error

                        (Some(url), None, id)
                    }
                    None => (None, None, None),
                };

                let thumbnail_view = ContentView::new(id, mime_type, url);

                let project_view = ProjectWithThumbnailView::new(
                    data.id,
                    data.metadata,
                    data.visible,
                    data.adult,
                    data.created_on,
                    data.updated_on,
                    Some(thumbnail_view),
                );
                vec.push(project_view);
                vec
            })
            .map(move |vec| vec)
            .await;
        Ok(ProjectsView::new(page, size, total_pages, result))
    }
}
