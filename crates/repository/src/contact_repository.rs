use app_core::{contact::contact_repository::IContactRepository, dto::contact_dto::ContactDto};
use app_error::Error;
use async_trait::async_trait;

use crate::{config::db::Db, entity::contact::Contact, error::to_error};

pub struct ContactRepository {
    pub db: Db,
}

impl ContactRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IContactRepository for ContactRepository {
    async fn get_contact(&self) -> Result<ContactDto, Error> {
        let sql = "SELECT * FROM contact FETCH FIRST ROW ONLY";
        let query = sqlx::query_as::<_, Contact>(sql);
        let contact = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;
        Ok(ContactDto::from(contact))
    }

    async fn update_contact(&self, id: i32, contact: &ContactDto) -> Result<ContactDto, Error> {
        let sql = "UPDATE contact SET description = $1 WHERE id = $2 RETURNING *";
        let query = sqlx::query_as::<_, Contact>(sql)
            .bind(contact.clone().description)
            .bind(id);
        let contact = query
            .fetch_one(&self.db)
            .await
            .map_err(|sqlx_error| to_error(sqlx_error, None))?;

        Ok(ContactDto::from(contact))
    }
}
