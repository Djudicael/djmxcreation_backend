use std::sync::Arc;

use app_core::{contact::contact_repository::IContactRepository, dto::contact_dto::ContactDto};
use app_error::Error;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use wasi_pg_client::Row;

use crate::{config::db::DatabaseConfig, entity::contact::Contact, error::to_error};

pub struct ContactRepository {
    config: Arc<DatabaseConfig>,
}

impl ContactRepository {
    pub fn new(config: Arc<DatabaseConfig>) -> Self {
        Self { config }
    }

    fn map_row_to_contact(row: &Row) -> Result<Contact, Error> {
        let description: Option<Value> = row
            .get_by_name::<Option<Value>>("description")
            .map_err(|e| to_error(e, None))?;

        let id: Uuid = row.get_by_name("id").map_err(|e| to_error(e, None))?;

        Ok(Contact::new(Some(id), description))
    }
}

#[async_trait]
impl IContactRepository for ContactRepository {
    async fn get_contact(&self) -> Result<ContactDto, Error> {
        let sql = "SELECT * FROM contact FETCH FIRST ROW ONLY";
        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let row = conn
            .query_one(sql)
            .await
            .map_err(|e| to_error(e, None))?
            .ok_or_else(|| Error::EntityNotFound("contact not found".to_string()))?;

        let contact = ContactRepository::map_row_to_contact(&row)?;
        Ok(ContactDto::from(contact))
    }

    async fn update_contact(&self, id: Uuid, contact: &ContactDto) -> Result<ContactDto, Error> {
        let sql = "UPDATE contact SET description = $1 WHERE id = $2 RETURNING *";
        let mut conn = self.config.connect().await.map_err(|e| to_error(e, None))?;
        let row = conn
            .query_params(sql, &[&contact.description, &id])
            .await
            .map_err(|e| to_error(e, None))?
            .into_rows()
            .into_iter()
            .next()
            .ok_or_else(|| Error::EntityNotFound(format!("contact not found for id: {}", id)))?;

        let contact = ContactRepository::map_row_to_contact(&row)?;
        Ok(ContactDto::from(contact))
    }
}
