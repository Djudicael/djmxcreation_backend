use app_core::home::{DynIHomeRepository, HomeSettings, IHomeService};
use app_error::Error;
use async_trait::async_trait;

pub struct HomeService {
    repository: DynIHomeRepository,
}

impl HomeService {
    pub fn new(repository: DynIHomeRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl IHomeService for HomeService {
    async fn get(&self, public: bool) -> Result<HomeSettings, Error> {
        self.repository.get(public).await
    }

    async fn save(&self, mut settings: HomeSettings) -> Result<HomeSettings, Error> {
        if settings.introduction.chars().count() > 500 {
            return Err(Error::InvalidInput(
                "The introduction must not exceed 500 characters.".into(),
            ));
        }
        settings.introduction = settings.introduction.trim().to_owned();
        self.repository.save(&settings).await
    }
}
