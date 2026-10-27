use crate::{config::db::DatabaseConfig, error::to_error};
use app_core::home::{HomeSettings, IHomeRepository};
use app_error::Error;
use async_trait::async_trait;
use std::sync::Arc;
use wasi_pg_client::Row;

// The same eligibility rules apply when saving and on every public read.
const ELIGIBLE: &str = "SELECT 1 FROM project p
    JOIN project_content_thumbnail t ON t.project_id = p.id
    WHERE p.visible IS TRUE AND p.adult IS FALSE
    AND (t.content->>'mime_type' LIKE 'image/%'
        OR (t.content->>'mime_type' IS NULL
            AND t.content->>'file_name' ~* '\\.(png|jpe?g|gif|webp|avif|svg|bmp)$'))";

pub struct HomeRepository {
    config: Arc<DatabaseConfig>,
}

impl HomeRepository {
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        Self { config }
    }

    fn from_row(row: &Row) -> Result<HomeSettings, Error> {
        Ok(HomeSettings {
            introduction: row
                .get_by_name("introduction")
                .map_err(|e| to_error(e, None))?,
            featured_project_id: row
                .get_by_name("featured_project_id")
                .map_err(|e| to_error(e, None))?,
        })
    }
}

#[async_trait]
impl IHomeRepository for HomeRepository {
    async fn get(&self, public: bool) -> Result<HomeSettings, Error> {
        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let sql = format!(
            "SELECT introduction,
            CASE WHEN NOT $1 OR EXISTS ({ELIGIBLE} AND p.id = h.featured_project_id)
            THEN featured_project_id ELSE NULL END AS featured_project_id
            FROM home_settings h WHERE singleton = TRUE"
        );
        let result = conn
            .query_params(&sql, &[&public])
            .await
            .map_err(|e| to_error(e, None))?;
        let row = result
            .into_rows()
            .into_iter()
            .next()
            .ok_or_else(|| Error::EntityNotFound("Home settings not found".into()))?;
        Self::from_row(&row)
    }

    async fn save(&self, settings: &HomeSettings) -> Result<HomeSettings, Error> {
        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let sql = format!(
            "UPDATE home_settings SET introduction = $1, featured_project_id = $2
            WHERE singleton = TRUE AND ($2::uuid IS NULL OR EXISTS ({ELIGIBLE} AND p.id = $2))
            RETURNING introduction, featured_project_id"
        );
        let result = conn
            .with_transaction(async |txn| {
                txn.query_params(
                    &sql,
                    &[&settings.introduction, &settings.featured_project_id],
                )
                .await
            })
            .await
            .map_err(|e| to_error(e, None))?;
        let row = result.into_rows().into_iter().next().ok_or_else(|| {
            Error::InvalidInput("Choose a published, non-adult project with an image cover.".into())
        })?;
        Self::from_row(&row)
    }
}
