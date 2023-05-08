use app_core::{
    dto::spotlight_dto::{self, SpotlightDto},
    spotlight::spotlight_repository::ISpotlightRepository,
};
use app_error::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{config::db::Db, entity::spotlight::Spotlight, error::to_error};

pub struct SpotlightRepository {
    db: Db,
}

impl SpotlightRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ISpotlightRepository for SpotlightRepository {
    async fn add_spotlight(&self, project_id: i32) -> Result<SpotlightDto, Error> {
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO spotlight (project_id, created_on) VALUES ($1, $2)
        RETURNING spotlight.id, spotlight.project_id, spotlight.adult, spotlight.metadata, spotlight.created_on, c.content AS thumbnail
        LEFT JOIN project_content_thumbnail c ON c.project_id = spotlight.project_id
        LEFT JOIN project_content ct ON ct.project_id = spotlight.project_id";

        let row = sqlx::query_as::<_, Spotlight>(sql)
            .bind(project_id)
            .bind(now_utc)
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        let spotlight_dto = SpotlightDto::from(row);

        Ok(spotlight_dto)
    }
    async fn get_spotlights(&self) -> Result<Vec<SpotlightDto>, Error> {
        let sql = "SELECT spotlight.id, spotlight.project_id, spotlight.adult, spotlight.metadata, spotlight.created_on, c.content AS thumbnail FROM spotlight 
        LEFT JOIN project_content_thumbnail c ON c.project_id = spotlight.project_id
        LEFT JOIN project_content ct ON ct.project_id = spotlight.project_id";

        let rows = sqlx::query_as::<_, Spotlight>(sql)
            .fetch_all(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        let spotlight_dto: Vec<SpotlightDto> =
            rows.iter().map(|s| SpotlightDto::from(s.clone())).collect();

        Ok(spotlight_dto)
    }
    async fn get_spotlight(&self, id: i32) -> Result<SpotlightDto, Error> {
        let sql = "SELECT spotlight.id, spotlight.project_id, spotlight.adult, spotlight.metadata, spotlight.created_on, c.content AS thumbnail FROM spotlight
        LEFT JOIN project_content_thumbnail c ON c.project_id = spotlight.project_id
        LEFT JOIN project_content ct ON ct.project_id = spotlight.project_id
        WHERE spotlight.id = $1";

        let row = sqlx::query_as::<_, Spotlight>(sql)
            .bind(id)
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        let spotlight_dto = SpotlightDto::from(row);

        Ok(spotlight_dto)
    }
    async fn delete_spotlight(&self, id: i32) -> Result<(), Error> {
        let sql = "DELETE FROM spotlight WHERE id = $1";

        sqlx::query(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        Ok(())
    }
}
