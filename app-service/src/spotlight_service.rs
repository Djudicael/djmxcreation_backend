use app_core::{
    dto::spotlight_dto::SpotlightDto,
    project::project_repository::DynIProjectRepository,
    spotlight::{
        spotlight_repository::DynISpotlightRepository, spotlight_service::ISpotlightService,
    },
    storage::storage_repository::DynIStorageRepository,
    view::{content_view::ContentView, spotlight_view::SpotlightView},
};
use app_error::Error;
use async_trait::async_trait;
use futures::{stream, FutureExt, StreamExt};

pub struct SpotlightService {
    pub spotlight_repository: DynISpotlightRepository,
    pub storage_repository: DynIStorageRepository,
    pub project_repository: DynIProjectRepository,
}

impl SpotlightService {
    pub fn new(
        spotlight_repository: DynISpotlightRepository,
        project_repository: DynIProjectRepository,
        storage_repository: DynIStorageRepository,
    ) -> Self {
        Self {
            spotlight_repository,
            storage_repository,
            project_repository,
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
}

#[async_trait]
impl ISpotlightService for SpotlightService {
    async fn add_spotlight(&self, project_id: i32) -> Result<SpotlightView, Error> {
        let _ = self
            .project_repository
            .get_project_by_id(project_id)
            .await?;
        let spotlight = self.spotlight_repository.add_spotlight(project_id).await?;
        self.to_spotlight_view(&spotlight).await
    }

    async fn get_spotlight(&self, id: i32) -> Result<SpotlightView, Error> {
        let spotlight = self.spotlight_repository.get_spotlight(id).await?;
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

    async fn delete_spotlight(&self, id: i32) -> Result<(), Error> {
        self.spotlight_repository.delete_spotlight(id).await?;
        Ok(())
    }
}
