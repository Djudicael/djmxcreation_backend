use crate::{config::db::Db, entity::about_me::AboutMe, error::to_error};
use app_core::{
    about_me::about_me_repository::IAboutMeRepository,
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
};
use app_error::Error;
use async_trait::async_trait;
use serde_json::json;

use sqlx::types::Json;
use sqlx::{Postgres, Transaction};

pub struct AboutMeRepository {
    db: Db,
}

impl AboutMeRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    // Helper function to handle transactions with error mapping
    async fn start_transaction<'a>(&'a self) -> Result<Transaction<'a, Postgres>, Error> {
        self.db.begin().await.map_err(|e| to_error(e, None))
    }
}

#[async_trait]
impl IAboutMeRepository for AboutMeRepository {
    async fn update_about_me(&self, id: i32, about: &AboutMeDto) -> Result<AboutMeDto, Error> {
        let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3 WHERE id = $4 RETURNING *";
        let AboutMeDto {
            first_name,
            last_name,
            description,
            ..
        } = about.clone();

        let about_me = sqlx::query_as::<_, AboutMe>(sql)
            .bind(first_name)
            .bind(last_name)
            .bind(description)
            .bind(id)
            .fetch_one(&self.db)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me(&self) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about LIMIT 1";
        let about_me = sqlx::query_as::<_, AboutMe>(sql)
            .fetch_one(&self.db)
            .await
            .map_err(|e| to_error(e, None))?;

        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me_by_id(&self, id: i32) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about WHERE id = $1";
        let about_me = sqlx::query_as::<_, AboutMe>(sql)
            .bind(id)
            .fetch_one(&self.db)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        Ok(AboutMeDto::from(about_me))
    }

    async fn update_photo(&self, id: i32, content: &ContentDto) -> Result<(), Error> {
        let mut tx = self.start_transaction().await?;
        let content_json = Json(json!(content));

        sqlx::query("UPDATE about SET photo = $1 WHERE id = $2")
            .bind(content_json)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        tx.commit()
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;
        Ok(())
    }

    async fn delete_about_me_photo(&self, id: i32) -> Result<(), Error> {
        let mut tx = self.start_transaction().await?;

        sqlx::query("UPDATE about SET photo = NULL WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        tx.commit()
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;
        Ok(())
    }
}
