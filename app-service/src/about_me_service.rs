use app_core::{
    about_me::{about_me_repository::DynIAboutMeRepository, about_me_service::IAboutMeService},
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
    storage::storage_repository::DynIStorageRepository,
    view::me_view::MeView,
};
use app_error::Error;
use async_trait::async_trait;

pub struct AboutMeService {
    pub about_me_repository: DynIAboutMeRepository,
    pub storage_repository: DynIStorageRepository,
}

impl AboutMeService {
    pub fn new(
        about_me_repository: DynIAboutMeRepository,
        storage_repository: DynIStorageRepository,
    ) -> Self {
        Self {
            about_me_repository,
            storage_repository,
        }
    }
}

#[async_trait]
impl IAboutMeService for AboutMeService {
    async fn about_me(&self) -> Result<MeView, Error> {
        let about_me = self.about_me_repository.get_about_me().await?;

        let content = about_me.clone().photo;

        let url = match content {
            Some(photo) => {
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await?;
                Some(url)
            }
            None => None,
        };
        let mut me = MeView::from(about_me);
        me.photo_url = url;
        Ok(me)
    }

    async fn update_me(&self, id: i32, about: &AboutMeDto) -> Result<MeView, Error> {
        let _ = self.about_me_repository.get_about_me_by_id(id).await?;
        let result = self.about_me_repository.update_about_me(id, about).await?;
        let content = result.clone().photo;
        let url = match content {
            Some(photo) => {
                let url = self
                    .storage_repository
                    .get_object_url(&photo.bucket_name, &photo.file_name)
                    .await?;
                Some(url)
            }
            None => None,
        };
        let mut me = MeView::from(result);
        me.photo_url = url;
        Ok(me)
    }

    async fn add_profile_picture(
        &self,
        id: i32,
        file_name: String,
        file: &[u8],
    ) -> Result<(), Error> {
        let me = self.about_me_repository.get_about_me_by_id(id).await?;
        let key = format!("{}/{}", "about", file_name);
        let previous_content = me.photo;
        let bucket = "portfolio";
        let content = ContentDto::new(None, bucket.to_owned(), key.clone(), None);
        self.storage_repository
            .upload_file(bucket, key.as_str(), file)
            .await?;
        self.about_me_repository.update_photo(id, &content).await?;

        // delete previous image from bucket
        if let Some(content) = previous_content {
            self.storage_repository
                .remove_object(&content.bucket_name, &content.file_name)
                .await?
        }

        Ok(())
    }

    async fn delete_photo(&self, id: i32) -> Result<(), Error> {
        let me = self.about_me_repository.get_about_me_by_id(id).await?;
        let previous_content = me.photo;
        self.about_me_repository.delete_about_me_photo(id).await?;

        if let Some(content) = previous_content {
            self.storage_repository
                .remove_object(&content.bucket_name, &content.file_name)
                .await?
        }

        Ok(())
    }
}
