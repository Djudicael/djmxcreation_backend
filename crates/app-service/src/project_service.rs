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
use futures::{stream, FutureExt, StreamExt};

pub struct ProjectService {
    pub project_repository: DynIProjectRepository,
    pub storage_repository: DynIStorageRepository,
    pub spotlight_repository: DynISpotlightRepository,
}

impl ProjectService {
    pub fn new(
        project_repository: DynIProjectRepository,
        storage_repository: DynIStorageRepository,
        spotlight_repository: DynISpotlightRepository,
    ) -> Self {
        Self {
            project_repository,
            storage_repository,
            spotlight_repository,
        }
    }

    async fn to_spotlight_view(&self, spotlight: &SpotlightDto) -> Result<SpotlightView, Error> {
        let thumbnail = match &spotlight.thumbnail {
            Some(photo) => {
                let id = photo.id;
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await
                    .ok();
                Some(ContentView::new(id, photo.mime_type.clone(), url))
            }
            None => None,
        };

        let spot_view = SpotlightView::new(
            spotlight.id,
            spotlight.project_id,
            spotlight.adult,
            spotlight.metadata.clone(),
            spotlight.created_on,
            thumbnail,
        );

        Ok(spot_view)
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
        println!("Creating project {metadata:?}");
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

    async fn add_thumbnail_to_project(
        &self,
        id: i32,
        content_id: i32,
    ) -> Result<ContentView, Error> {
        let _ = self.project_repository.get_project_by_id(id).await?;
        let project_contents = self
            .project_repository
            .get_projects_content_by_id(id, content_id)
            .await?;

        let thumbnail = project_contents.content;

        match thumbnail {
            Some(photo) => {
                let mut content = photo.clone();
                content.id = project_contents.id;
                let thumbnail_saved = self
                    .project_repository
                    .add_project_thumbnail(id, &content)
                    .await?;
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await?;

                let content_view = ContentView::new(thumbnail_saved.id, photo.mime_type, Some(url));
                Ok(content_view)
            }
            None => Err(Error::ContentNotFoundButWasSave(
                "Content was found but no images was associated".to_string(),
            )),
        }
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

        // println!("Project entity: {:#?}", project_entity);

        let mut project_view: ProjectView = ProjectView::from(project_entity.clone());
        let mut contents: Vec<ContentView> = vec![];

        for content_dto in project_entity.contents {
            let content = content_dto.content;
            if let Some(photo) = content {
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await
                    .ok();

                let content_view = ContentView::new(content_dto.id, photo.mime_type, url);
                contents.push(content_view);
            };
        }

        let thumbnail = match project_entity.thumbnail {
            Some(photo) => {
                let id = photo.id;
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await
                    .ok();
                Some(ContentView::new(id, photo.mime_type, url))
            }
            None => None,
        };

        project_view.contents = contents.to_vec();
        project_view.thumbnail = thumbnail;

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

        let thumbnail = match __self
            .project_repository
            .get_thumbnail_by_id(project_id, content_id)
            .await
        {
            Ok(thumb) => thumb,
            Err(_) => None,
        };

        if let Some(thumbnail) = thumbnail {
            if let Some(id) = thumbnail.id {
                self.project_repository
                    .delete_thumbnail_by_id(project_id, id)
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_portfolio_projects(&self) -> Result<Vec<ProjectView>, Error> {
        let projects = self.project_repository.get_projects().await?;

        let result = stream::iter(projects)
            .fold(Vec::new(), |mut vec, data| async move {
                let contents = self
                    .to_contents(&data.contents)
                    .await
                    .expect("List of contents");

                let thumbnail_view = match data.clone().thumbnail {
                    Some(photo) => {
                        let id = photo.id;
                        let url = self
                            .storage_repository
                            .get_object_url(&photo.bucket_name, &photo.file_name)
                            .await
                            .ok();

                        Some(ContentView::new(id, photo.mime_type, url))
                    }
                    None => None,
                };
                let project_view: ProjectView = ProjectView::new()
                    .id(data.id)
                    .description(data.description)
                    .metadata(data.metadata)
                    .visible(data.visible)
                    .adult(data.adult)
                    .created_on(data.created_on)
                    .updated_on(data.updated_on)
                    .thumbnail(thumbnail_view)
                    .contents(contents)
                    .build();
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
                let thumbnail_view = match content {
                    Some(photo) => {
                        let id = photo.id;
                        let url = self
                            .storage_repository
                            .get_object_url(&photo.bucket_name, &photo.file_name)
                            .await
                            .ok();

                        Some(ContentView::new(id, photo.mime_type, url))
                    }
                    None => None,
                };

                let project_view = ProjectWithThumbnailView::new(
                    data.id,
                    data.metadata,
                    data.visible,
                    data.adult,
                    data.created_on,
                    data.updated_on,
                    thumbnail_view,
                );
                vec.push(project_view);
                vec
            })
            .map(move |vec| vec)
            .await;
        Ok(ProjectsView::new(page, size, total_pages, result))
    }

    async fn add_spotlight(&self, project_id: i32) -> Result<SpotlightView, Error> {
        let _ = self
            .project_repository
            .get_project_by_id(project_id)
            .await?;
        let spotlight = self.spotlight_repository.add_spotlight(project_id).await?;
        self.to_spotlight_view(&spotlight).await
    }

    async fn get_spotlight(&self, spotlight_id: i32) -> Result<SpotlightView, Error> {
        let spotlight = self
            .spotlight_repository
            .get_spotlight(spotlight_id)
            .await?;
        self.to_spotlight_view(&spotlight).await
    }

    async fn get_spotlights(&self) -> Result<Vec<SpotlightView>, Error> {
        let spotlights = self.spotlight_repository.get_spotlights().await?;
        let result = stream::iter(spotlights)
            .fold(Vec::new(), |mut vec, data| async move {
                let spotlight_view = self.to_spotlight_view(&data).await.expect("msg");
                vec.push(spotlight_view);
                vec
            })
            .map(move |vec| vec)
            .await;
        Ok(result)
    }

    async fn delete_spotlight(&self, spotlight_id: i32) -> Result<(), Error> {
        self.spotlight_repository
            .delete_spotlight(spotlight_id)
            .await?;
        Ok(())
    }
}
