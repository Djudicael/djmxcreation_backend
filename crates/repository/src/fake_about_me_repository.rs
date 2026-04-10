use std::sync::{Arc, Mutex};

use app_core::{
    about_me::about_me_repository::{DynIAboutMeRepository, IAboutMeRepository},
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
};
use app_error::Error;
use async_trait::async_trait;
use uuid::Uuid;

struct State {
    about_me: AboutMeDto,
    about_me_by_id: AboutMeDto,
    updated_photo: Mutex<Option<(Uuid, ContentDto)>>,
    deleted_photo_ids: Mutex<Vec<Uuid>>,
}

struct FakeAboutMeRepository {
    state: Arc<State>,
}

pub struct AboutMeRepositoryProbe {
    state: Arc<State>,
}

impl AboutMeRepositoryProbe {
    pub fn last_updated_photo(&self) -> Option<(Uuid, ContentDto)> {
        self.state
            .updated_photo
            .lock()
            .expect("about-me photo lock poisoned")
            .clone()
    }

    pub fn deleted_photo_ids(&self) -> Vec<Uuid> {
        self.state
            .deleted_photo_ids
            .lock()
            .expect("about-me delete lock poisoned")
            .clone()
    }
}

#[async_trait]
impl IAboutMeRepository for FakeAboutMeRepository {
    async fn update_about_me(&self, _id: Uuid, _about: &AboutMeDto) -> Result<AboutMeDto, Error> {
        Ok(self.state.about_me_by_id.clone())
    }

    async fn get_about_me(&self) -> Result<AboutMeDto, Error> {
        Ok(self.state.about_me.clone())
    }

    async fn get_about_me_by_id(&self, _id: Uuid) -> Result<AboutMeDto, Error> {
        Ok(self.state.about_me_by_id.clone())
    }

    async fn update_photo(&self, id: Uuid, content: &ContentDto) -> Result<(), Error> {
        self.state
            .updated_photo
            .lock()
            .expect("about-me photo lock poisoned")
            .replace((id, content.clone()));
        Ok(())
    }

    async fn delete_about_me_photo(&self, id: Uuid) -> Result<(), Error> {
        self.state
            .deleted_photo_ids
            .lock()
            .expect("about-me delete lock poisoned")
            .push(id);
        Ok(())
    }
}

pub fn create_about_me_repository(
    about_me: AboutMeDto,
    about_me_by_id: AboutMeDto,
) -> (DynIAboutMeRepository, AboutMeRepositoryProbe) {
    let state = Arc::new(State {
        about_me,
        about_me_by_id,
        updated_photo: Mutex::new(None),
        deleted_photo_ids: Mutex::new(Vec::new()),
    });

    let repository: DynIAboutMeRepository = Arc::new(FakeAboutMeRepository {
        state: state.clone(),
    });

    (repository, AboutMeRepositoryProbe { state })
}
