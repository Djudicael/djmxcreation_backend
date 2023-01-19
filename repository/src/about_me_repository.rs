use crate::{config::db::Db, entity::about_me::AboutMe, error::to_error};

use app_core::{
    about_me::about_me_repository::IAboutMeRepository,
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
};

use app_error::Error;

use async_trait::async_trait;
use serde_json::json;
use sqlx::types::Json;

pub struct AboutMeRepository {
    db: Db,
}

impl AboutMeRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IAboutMeRepository for AboutMeRepository {
    async fn update_about_me(&self, id: i32, about: &AboutMeDto) -> Result<AboutMeDto, Error> {
        let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3 WHERE id = $4 RETURNING *";
        // let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3, photo = $4 WHERE id = $5 RETURNING *";
        let query = sqlx::query_as::<_, AboutMe>(sql)
            .bind(about.clone().first_name)
            .bind(about.clone().last_name)
            .bind(about.clone().description)
            .bind(id);
        let about_me = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me(&self) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about FETCH FIRST ROW ONLY";
        let query = sqlx::query_as::<_, AboutMe>(sql);
        let about_me = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me_by_id(&self, id: i32) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about where id = $1 FETCH FIRST ROW ONLY";
        let query = sqlx::query_as::<_, AboutMe>(sql).bind(id);
        let about_me = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;

        Ok(AboutMeDto::from(about_me))
    }

    async fn update_photo(&self, id: i32, content: &ContentDto) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;

        let content_json = Json(json!(content));

        sqlx::query("UPDATE about SET photo = $1 WHERE id = $2 ")
            .bind(content_json)
            .bind(id)
            .execute(&mut tx)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;

        tx.commit()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;

        Ok(())
    }

    async fn delete_about_me_photo(&self, id: i32) -> Result<(), Error> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;
        sqlx::query("UPDATE about SET photo = NULL WHERE id = $1")
            .bind(id)
            .execute(&mut tx)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;
        tx.commit()
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, Some(id.to_string())))?;
        Ok(())
    }
}
