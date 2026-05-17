use std::sync::Arc;

use app_core::{
    dto::spotlight_dto::SpotlightDto, spotlight::spotlight_repository::ISpotlightRepository,
};
use app_error::Error;
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use wasi_pg_client::Row;

use crate::{config::db::DatabaseConfig, entity::spotlight::Spotlight, error::to_error};

pub struct SpotlightRepository {
    config: Arc<DatabaseConfig>,
}

impl SpotlightRepository {
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        Self { config }
    }

    fn map_row_to_spotlight(row: &Row) -> Result<Spotlight, Error> {
        let thumbnail: Option<Value> = row
            .get_by_name::<Option<Value>>("thumbnail")
            .map_err(|e| to_error(e, None))?;
        let metadata: Option<Value> = row
            .get_by_name::<Option<Value>>("metadata")
            .map_err(|e| to_error(e, None))?;

        let created_on: DateTime<Utc> = row
            .get_by_name("created_on")
            .map_err(|e| to_error(e, None))?;

        let id: Uuid = row.get_by_name("id").map_err(|e| to_error(e, None))?;
        let project_id: Uuid = row
            .get_by_name("project_id")
            .map_err(|e| to_error(e, None))?;
        let adult: bool = row.get_by_name("adult").map_err(|e| to_error(e, None))?;

        Ok(Spotlight {
            id: Some(id),
            project_id,
            adult,
            metadata,
            created_on: Some(created_on),
            thumbnail,
        })
    }
}

#[async_trait]
impl ISpotlightRepository for SpotlightRepository {
    async fn add_spotlight(&self, project_id: Uuid) -> Result<SpotlightDto, Error> {
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "WITH inserted AS (
        INSERT INTO project_spotlight (project_id, created_on)
        VALUES ($1, $2)
        RETURNING id, project_id, created_on
    )
    SELECT
        inserted.id,
        inserted.project_id,
        p.adult,
        p.metadata,
        inserted.created_on,
        c.content AS thumbnail
    FROM inserted
    LEFT JOIN project p ON p.id = inserted.project_id
    LEFT JOIN project_content_thumbnail c ON c.project_id = inserted.project_id";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let spotlight_dto = conn
            .with_transaction(async |txn| {
                let row = txn
                    .query_params(sql, &[&project_id, &now_utc])
                    .await?
                    .into_rows()
                    .into_iter()
                    .next()
                    .ok_or_else(|| wasi_pg_client::PgError::UnexpectedNull {
                        column: "id".to_string(),
                    })?;

                let spotlight = Self::map_row_to_spotlight(&row).map_err(|e| {
                    wasi_pg_client::PgError::TypeConversion(
                        wasi_pg_client::types::Error::Conversion(e.to_string()),
                    )
                })?;
                Ok(SpotlightDto::from(spotlight))
            })
            .await
            .map_err(|e| to_error(e, None))?;

        Ok(spotlight_dto)
    }

    async fn get_spotlights(&self) -> Result<Vec<SpotlightDto>, Error> {
        let sql = "SELECT
        ps.id,
        ps.project_id,
        p.adult,
        p.metadata,
        ps.created_on,
        c.content AS thumbnail
    FROM project_spotlight ps
    LEFT JOIN project p ON p.id = ps.project_id
    LEFT JOIN project_content_thumbnail c ON c.project_id = ps.project_id";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;

        let result = conn
            .query(sql)
            .await
            .map_err(|error| to_error(error, None))?;

        let spotlights = result
            .iter()
            .map(|row| Self::map_row_to_spotlight(row))
            .map(|spotlight| spotlight.map(SpotlightDto::from))
            .collect::<Result<Vec<SpotlightDto>, Error>>()?;

        Ok(spotlights)
    }

    async fn get_spotlight(&self, id: Uuid) -> Result<Option<SpotlightDto>, Error> {
        let sql = "SELECT
        ps.id,
        ps.project_id,
        p.adult,
        p.metadata,
        ps.created_on,
        c.content AS thumbnail
    FROM project_spotlight ps
    LEFT JOIN project p ON p.id = ps.project_id
    LEFT JOIN project_content_thumbnail c ON c.project_id = ps.project_id
    WHERE ps.id = $1";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;

        let row = conn
            .query_params(sql, &[&id])
            .await
            .map_err(|error| {
                to_error(
                    error,
                    Some(format!("Failed to fetch spotlight with id: {}", id)),
                )
            })?
            .into_rows()
            .into_iter()
            .next();

        match row {
            Some(row) => {
                let spotlight = Self::map_row_to_spotlight(&row)?;
                Ok(Some(SpotlightDto::from(spotlight)))
            }
            None => Ok(None),
        }
    }

    async fn delete_spotlight(&self, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project_spotlight WHERE id = $1";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.execute_params(sql, &[&id])
            .await
            .map_err(|error| to_error(error, Some(id.to_string())))?;

        Ok(())
    }
}
