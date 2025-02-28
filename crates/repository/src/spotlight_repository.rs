use std::{future::Future, pin::Pin, sync::Arc, time::SystemTime};

use app_core::{
    dto::spotlight_dto::SpotlightDto, spotlight::spotlight_repository::ISpotlightRepository,
};
use app_error::Error;
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_postgres::{Row, Transaction};
use uuid::Uuid;

use crate::{
    config::db::ClientV2,
    entity::spotlight::Spotlight,
    error::{handle_serde_json_error, handle_uuid_error, to_error},
};

pub struct SpotlightRepository {
    client: Arc<Mutex<ClientV2>>,
}

impl SpotlightRepository {
    pub fn new(client: Arc<Mutex<ClientV2>>) -> Self {
        Self { client }
    }

    async fn with_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(
            &'a Transaction<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>,
    {
        let mut client = self.client.lock().await;
        let transaction = client.transaction().await.map_err(|e| to_error(e, None))?;
        let result = f(&transaction).await?;
        transaction.commit().await.map_err(|e| to_error(e, None))?;
        Ok(result)
    }

    fn map_row_to_spotlight(row: &Row) -> Result<Spotlight, Error> {
        let thumbnail: Option<Value> = row
            .get::<_, Option<String>>(5)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let metadata: Option<serde_json::Value> = row
            .get::<_, Option<String>>(3)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;

        let created_on: Option<SystemTime> = row.get(4);
        let created_on: Option<DateTime<Utc>> = created_on.map(|time| time.into());
        let id = Uuid::parse_str(row.get(0)).map_err(|e| handle_uuid_error(e))?;
        let project_id = Uuid::parse_str(row.get(1)).map_err(|e| handle_uuid_error(e))?;

        Ok(Spotlight {
            id: Some(id),
            project_id,
            adult: row.get(2),
            metadata,
            created_on,
            thumbnail,
        })
    }
}

#[async_trait]
impl ISpotlightRepository for SpotlightRepository {
    async fn add_spotlight(&self, project_id: Uuid) -> Result<SpotlightDto, Error> {
        let now_utc: DateTime<Utc> = Utc::now();
        let sql = "INSERT INTO spotlight (project_id, created_on) VALUES ($1, $2)
        RETURNING spotlight.id, spotlight.project_id, spotlight.adult, spotlight.metadata, spotlight.created_on, c.content AS thumbnail
        LEFT JOIN project_content_thumbnail c ON c.project_id = spotlight.project_id
        LEFT JOIN project_content ct ON ct.project_id = spotlight.project_id";

        let spotlight_dto = self
            .with_transaction(|tx| {
                Box::pin(async move {
                    let row = tx
                        .query_one(sql, &[&project_id.to_string(), &now_utc.to_string()])
                        .await
                        .map_err(|error| to_error(error, None))?;

                    let spotlight = Self::map_row_to_spotlight(&row)?;

                    Ok(SpotlightDto::from(spotlight))
                })
            })
            .await?;

        Ok(spotlight_dto)
    }
    async fn get_spotlights(&self) -> Result<Vec<SpotlightDto>, Error> {
        let sql = "SELECT spotlight.id, spotlight.project_id, spotlight.adult, spotlight.metadata, spotlight.created_on, c.content AS thumbnail FROM spotlight 
        LEFT JOIN project_content_thumbnail c ON c.project_id = spotlight.project_id
        LEFT JOIN project_content ct ON ct.project_id = spotlight.project_id";

        let client = self.client.lock().await;

        let rows = client
            .query(sql, &[])
            .await
            .map_err(|error| to_error(error, None))?;

        let spotlights = rows
            .iter()
            .map(|row| Self::map_row_to_spotlight(row))
            .map(|spotlight| spotlight.map(SpotlightDto::from))
            .collect::<Result<Vec<SpotlightDto>, Error>>()?;

        Ok(spotlights)
    }
    async fn get_spotlight(&self, id: Uuid) -> Result<SpotlightDto, Error> {
        let sql = "SELECT spotlight.id, spotlight.project_id, spotlight.adult, spotlight.metadata, spotlight.created_on, c.content AS thumbnail FROM spotlight
        LEFT JOIN project_content_thumbnail c ON c.project_id = spotlight.project_id
        LEFT JOIN project_content ct ON ct.project_id = spotlight.project_id
        WHERE spotlight.id = $1";

        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[&id.to_string()])
            .await
            .map_err(|error| to_error(error, None))?;
        let spotlight = Self::map_row_to_spotlight(&row)?;

        let spotlight_dto = SpotlightDto::from(spotlight);

        Ok(spotlight_dto)
    }
    async fn delete_spotlight(&self, id: Uuid) -> Result<(), Error> {
        let sql = "DELETE FROM spotlight WHERE id = $1";

        let client = self.client.lock().await;
        client
            .execute(sql, &[&id.to_string()])
            .await
            .map_err(|error| to_error(error, Some(id.to_string())))?;

        Ok(())
    }
}
