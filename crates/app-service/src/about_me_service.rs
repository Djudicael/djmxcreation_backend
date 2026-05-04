use app_core::{
    about_me::{about_me_repository::DynIAboutMeRepository, about_me_service::IAboutMeService},
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
    storage::storage_repository::DynIStorageRepository,
    view::me_view::MeView,
};
use app_error::Error;
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

pub struct AboutMeService {
    pub about_me_repository: DynIAboutMeRepository,
    pub storage_repository: DynIStorageRepository,
    /// Name of the S3 bucket where all media is stored.
    bucket: String,
}

impl AboutMeService {
    pub fn new(
        about_me_repository: DynIAboutMeRepository,
        storage_repository: DynIStorageRepository,
        bucket: String,
    ) -> Self {
        Self {
            about_me_repository,
            storage_repository,
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
}

#[async_trait]
impl IAboutMeService for AboutMeService {
    async fn about_me(&self) -> Result<MeView, Error> {
        let about_me = self.about_me_repository.get_about_me().await?;
        let url = match &about_me.photo {
            Some(photo) => self.resolve_url(&photo.file_name).await,
            None => None,
        };
        let mut me = MeView::from(about_me);
        me.photo_url = url;
        Ok(me)
    }

    async fn update_me(&self, id: Uuid, about: &AboutMeDto) -> Result<MeView, Error> {
        let _ = self.about_me_repository.get_about_me_by_id(id).await?;
        let result = self.about_me_repository.update_about_me(id, about).await?;
        let url = match &result.photo {
            Some(photo) => self.resolve_url(&photo.file_name).await,
            None => None,
        };
        let mut me = MeView::from(result);
        me.photo_url = url;
        Ok(me)
    }

    async fn add_profile_picture(
        &self,
        id: Uuid,
        file_name: String,
        file: &[u8],
    ) -> Result<(), Error> {
        let me = self.about_me_repository.get_about_me_by_id(id).await?;
        // Store profile pictures in an "about/" sub-path within the bucket.
        let key = format!("about/{file_name}");
        let previous_content = me.photo;
        let content = ContentDto::new(None, self.bucket.clone(), key.clone(), None);
        self.storage_repository
            .upload_file(&self.bucket, &key, file)
            .await?;
        self.about_me_repository.update_photo(id, &content).await?;

        // Delete the old photo after successfully saving the new one.
        if let Some(old) = previous_content {
            if let Err(e) = self
                .storage_repository
                .remove_object(&self.bucket, &old.file_name)
                .await
            {
                warn!(file = %old.file_name, error = ?e, "failed to delete old profile photo");
            }
        }

        Ok(())
    }

    async fn delete_photo(&self, id: Uuid) -> Result<(), Error> {
        let me = self.about_me_repository.get_about_me_by_id(id).await?;
        let previous_content = me.photo;
        self.about_me_repository.delete_about_me_photo(id).await?;

        if let Some(old) = previous_content {
            if let Err(e) = self
                .storage_repository
                .remove_object(&self.bucket, &old.file_name)
                .await
            {
                warn!(file = %old.file_name, error = ?e, "failed to delete profile photo from storage");
            }
        }

        Ok(())
    }
}
