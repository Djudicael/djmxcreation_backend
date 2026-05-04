use std::sync::Arc;

use uuid::Uuid;

use crate::{config::db::DatabaseConfig, entity::about_me::AboutMe, error::to_error};
use app_core::{
    about_me::about_me_repository::IAboutMeRepository,
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
};
use app_error::Error;
use async_trait::async_trait;
use serde_json::{Value, json};
use wasi_pg_client::Row;

pub struct AboutMeRepository {
    config: Arc<DatabaseConfig>,
}

/// Extract an optional JSON column from a row.
fn json_opt(row: &Row, col: &str) -> Result<Option<Value>, Error> {
    row.get_by_name::<Option<Value>>(col)
        .map_err(|e| to_error(e, None))
}

/// Extract a required column value from a row.
fn col<T: wasi_pg_client::pg_types::FromSql>(row: &Row, col_name: &str) -> Result<T, Error> {
    row.get_by_name(col_name).map_err(|e| to_error(e, None))
}

impl AboutMeRepository {
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        Self { config }
    }

    fn map_row_to_about_me(row: &Row) -> Result<AboutMe, Error> {
        Ok(AboutMe::new(
            Some(col(row, "id")?),
            col(row, "first_name")?,
            col(row, "last_name")?,
            json_opt(row, "description")?,
            json_opt(row, "photo")?,
        ))
    }
}

#[async_trait]
impl IAboutMeRepository for AboutMeRepository {
    async fn update_about_me(&self, id: Uuid, about: &AboutMeDto) -> Result<AboutMeDto, Error> {
        let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3 WHERE id = $4 RETURNING *";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let result = conn
            .with_transaction(async |txn| {
                txn.query_params(
                    sql,
                    &[&about.first_name, &about.last_name, &about.description, &id],
                )
                .await
            })
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        let row =
            result.into_rows().into_iter().next().ok_or_else(|| {
                Error::EntityNotFound(format!("about_me not found for id: {}", id))
            })?;

        let about_me = Self::map_row_to_about_me(&row)?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me(&self) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about LIMIT 1";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let row = conn
            .query_one(sql)
            .await
            .map_err(|e| to_error(e, None))?
            .ok_or_else(|| Error::EntityNotFound("about_me not found".to_string()))?;

        let about_me = Self::map_row_to_about_me(&row)?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me_by_id(&self, id: Uuid) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about WHERE id = $1";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let row = conn
            .query_params(sql, &[&id])
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?
            .into_rows()
            .into_iter()
            .next()
            .ok_or_else(|| Error::EntityNotFound(format!("about_me not found for id: {}", id)))?;

        let about_me = Self::map_row_to_about_me(&row)?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn update_photo(&self, id: Uuid, content: &ContentDto) -> Result<(), Error> {
        let content_json = json!(content);
        let sql = "UPDATE about SET photo = $1 WHERE id = $2";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.with_transaction(async |txn| txn.execute_params(sql, &[&content_json, &id]).await)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        Ok(())
    }

    async fn delete_about_me_photo(&self, id: Uuid) -> Result<(), Error> {
        let sql = "UPDATE about SET photo = NULL WHERE id = $1";

        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        conn.with_transaction(async |txn| txn.execute_params(sql, &[&id]).await)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        Ok(())
    }
}
