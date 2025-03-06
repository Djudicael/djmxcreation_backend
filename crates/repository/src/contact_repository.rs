use std::sync::Arc;

use app_core::{contact::contact_repository::IContactRepository, dto::contact_dto::ContactDto};
use app_error::Error;
use async_trait::async_trait;
use deadpool_postgres::PoolError;
use serde_json::Value;

use tokio_postgres::{types::Json, Row};
use uuid::Uuid;

use crate::{config::db::DatabasePool, entity::contact::Contact, error::to_error};

pub struct ContactRepository {
    client: Arc<DatabasePool>,
}

impl ContactRepository {
    pub fn new(client: Arc<DatabasePool>) -> Self {
        Self { client }
    }

    fn map_row_to_contact(row: &Row) -> Result<Contact, Error> {
        let description: Option<Value> = row
            .try_get::<_, Option<Json<Value>>>("description")
            .map_err(|e| to_error(PoolError::Backend(e), None))?
            .map(|json| json.0);

        let id: Uuid = row
            .try_get("id")
            .map_err(|e| to_error(PoolError::Backend(e), None))?;

        Ok(Contact::new(Some(id), description))
    }
}

#[async_trait]
impl IContactRepository for ContactRepository {
    async fn get_contact(&self) -> Result<ContactDto, Error> {
        let sql = "SELECT * FROM contact FETCH FIRST ROW ONLY";
        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        let row = client
            .query_one(sql, &[])
            .await
            .map_err(|sql_error| to_error(PoolError::Backend(sql_error), None))?;
        let contact = ContactRepository::map_row_to_contact(&row)?;
        Ok(ContactDto::from(contact))
    }

    async fn update_contact(&self, id: Uuid, contact: &ContactDto) -> Result<ContactDto, Error> {
        let sql = "UPDATE contact SET description = $1 WHERE id = $2 RETURNING *";
        let description = contact.description.as_ref().map(|v| v.to_string());
        let client = self
            .client
            .get_client()
            .await
            .map_err(|e| to_error(e, None))?;
        let row = client
            .query_one(sql, &[&Json(description), &id])
            .await
            .map_err(|sql_error| to_error(PoolError::Backend(sql_error), None))?;
        let contact = ContactRepository::map_row_to_contact(&row)?;

        Ok(ContactDto::from(contact))
    }
}
