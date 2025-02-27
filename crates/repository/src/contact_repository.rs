use std::{future::Future, pin::Pin, sync::Arc};

use app_core::{contact::contact_repository::IContactRepository, dto::contact_dto::ContactDto};
use app_error::Error;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio_postgres::{Row, Transaction};
use uuid::Uuid;

use crate::{
    config::db::ClientV2,
    entity::contact::Contact,
    error::{handle_serde_json_error, handle_uuid_error, to_error},
};

pub struct ContactRepository {
    client: Arc<Mutex<ClientV2>>,
}

impl ContactRepository {
    pub fn new(client: Arc<Mutex<ClientV2>>) -> Self {
        Self { client }
    }

    fn map_row_to_contact(row: &Row) -> Result<Contact, Error> {
        let description: Option<Value> = row
            .get::<_, Option<String>>(1)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;
        let id = Uuid::parse_str(row.get(0)).map_err(|e| handle_uuid_error(e))?;
        Ok(Contact::new(Some(id), description))
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
}

#[async_trait]
impl IContactRepository for ContactRepository {
    async fn get_contact(&self) -> Result<ContactDto, Error> {
        let sql = "SELECT * FROM contact FETCH FIRST ROW ONLY";
        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[])
            .await
            .map_err(|sql_error| to_error(sql_error, None))?;
        let contact = ContactRepository::map_row_to_contact(&row)?;
        Ok(ContactDto::from(contact))
    }

    async fn update_contact(&self, id: i32, contact: &ContactDto) -> Result<ContactDto, Error> {
        let sql = "UPDATE contact SET description = $1 WHERE id = $2 RETURNING *";
        let description = contact.description.as_ref().map(|v| v.to_string());
        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[&description, &id])
            .await
            .map_err(|sql_error| to_error(sql_error, None))?;
        let contact = ContactRepository::map_row_to_contact(&row)?;

        Ok(ContactDto::from(contact))
    }
}
