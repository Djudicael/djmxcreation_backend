use std::{future::Future, pin::Pin, sync::Arc, time::SystemTime};

use app_core::{
    dto::spotlight_dto::SpotlightDto, spotlight::spotlight_repository::ISpotlightRepository,
};
use app_error::Error;
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use deadpool_postgres::PoolError;
use serde_json::Value;

use tokio_postgres::{types::Json, Row, Transaction};
use uuid::Uuid;

use crate::{config::db::DatabasePool, entity::spotlight::Spotlight, error::to_error};

pub struct SpotlightRepository {
    client: Arc<DatabasePool>,
}

impl SpotlightRepository {
    pub fn new(client: Arc<DatabasePool>) -> Self {
        Self { client }
    }

    async fn with_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(
            &'a Transaction<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>,
    {
        let mut client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;

        let transaction = client
            .build_transaction()
            .start()
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        let result = f(&transaction).await?;

        transaction
            .commit()
            .await
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        Ok(result)
    }

    fn map_row_to_spotlight(row: &Row) -> Result<Spotlight, Error> {
        let thumbnail: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("thumbnail")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);
        let metadata: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("metadata")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let created_on: SystemTime = row
            .try_get("created_on")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let created_on: DateTime<Utc> = created_on.into();
        let id: Uuid = row
            .try_get("id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;
        let project_id = row
            .try_get("project_id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        Ok(Spotlight {
            id: Some(id),
            project_id,
            adult: row.get(2),
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

        let spotlight_dto = self
            .with_transaction(|tx| {
                Box::pin(async move {
                    let row = tx
                        .query_one(sql, &[&project_id, &now_utc])
                        .await
                        .map_err(|error| to_error(PoolError::Backend(error), None))?;

                    let spotlight = Self::map_row_to_spotlight(&row)?;

                    Ok(SpotlightDto::from(spotlight))
                })
            })
            .await?;

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

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;

        let rows = client
            .query(sql, &[])
            .await
            .map_err(|error| to_error(PoolError::Backend(error), None))?;

        let spotlights = rows
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

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;

        match client.query_opt(sql, &[&id]).await.map_err(|error| {
            to_error(
                PoolError::Backend(error),
                Some(format!("Failed to fetch spotlight with id: {}", id)),
            )
        })? {
            Some(row) => {
                let spotlight = Self::map_row_to_spotlight(&row)?;
                Ok(Some(SpotlightDto::from(spotlight)))
            }
            None => Ok(None),
        }
    }

    async fn delete_spotlight(&self, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM project_spotlight WHERE id = $1";

        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        client
            .execute(sql, &[&id])
            .await
            .map_err(|error| to_error(PoolError::Backend(error), Some(id.to_string())))?;

        Ok(())
    }
}
