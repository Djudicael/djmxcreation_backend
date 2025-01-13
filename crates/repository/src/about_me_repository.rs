use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    config::db::ClientV2,
    entity::about_me::AboutMe,
    error::{handle_serde_json_error, to_error},
};
use app_core::{
    about_me::about_me_repository::IAboutMeRepository,
    dto::{about_me_dto::AboutMeDto, content_dto::ContentDto},
};
use app_error::Error;
use async_trait::async_trait;
use serde_json::{json, Value};

use tokio_postgres::{Row, Transaction};

pub struct AboutMeRepository {
    client: Arc<Mutex<ClientV2>>,
}

impl AboutMeRepository {
    pub fn new(client: ClientV2) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    // Helper to map a database row to AboutMe
    fn map_row_to_about_me(row: &Row) -> Result<AboutMe, Error> {
        // Try to parse the JSON strings in the 3rd and 4th columns and return the result
        let description: Option<Value> = row
            .get::<_, Option<String>>(3)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;

        let photo: Option<Value> = row
            .get::<_, Option<String>>(4)
            .map(|s| serde_json::from_str(&s).map_err(|e| handle_serde_json_error(e)))
            .transpose()?;

        // Return the AboutMe object with the parsed data
        Ok(AboutMe::new(
            row.get(0), // Assuming id is at index 0
            row.get(1), // Assuming first_name is at index 1
            row.get(2), // Assuming last_name is at index 2
            description,
            photo,
        ))
    }

    async fn with_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(Transaction<'_>) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send>> + Send, // Expect an async closure
    {
        let mut client = self.client.lock().await; // Acquire the lock
        let transaction = client.transaction().await.map_err(|e| to_error(e, None))?;
        let result = f(transaction).await?; // Execute the async closure
        Ok(result) // Return the result
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

        let client = self.client.lock().await;
        let stmt = client
            .prepare(sql)
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        let row = client
            .query_one(
                &stmt,
                &[
                    &first_name,
                    &last_name,
                    &description.map(|v| v.to_string()),
                    &id,
                ],
            )
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        let about_me = Self::map_row_to_about_me(&row)?;

        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me(&self) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about LIMIT 1";
        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[])
            .await
            .map_err(|e| to_error(e, None))?;

        let about_me = Self::map_row_to_about_me(&row)?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn get_about_me_by_id(&self, id: i32) -> Result<AboutMeDto, Error> {
        let sql = "SELECT * FROM about WHERE id = $1";
        let client = self.client.lock().await;
        let row = client
            .query_one(sql, &[&id])
            .await
            .map_err(|e| to_error(e, Some(id.to_string())))?;

        let about_me = Self::map_row_to_about_me(&row)?;
        Ok(AboutMeDto::from(about_me))
    }

    async fn update_photo(&self, id: i32, content: &ContentDto) -> Result<(), Error> {
        let content_json = json!(content);
        let sql = "UPDATE about SET photo = $1 WHERE id = $2";

        // Use the `with_transaction` method and return a Result from the closure
        self.with_transaction(|mut tx| async move {
            tx.execute(sql, &[&content_json.to_string(), &id])
                .await
                .map_err(|e| to_error(e, Some(id.to_string())))?;
            Ok(()) // Ensure the closure returns a Result
        })
        .await // Await the result of `with_transaction`
    }

    async fn delete_about_me_photo(&self, id: i32) -> Result<(), Error> {
        let mut tx = self.start_transaction().await?;

        self.with_transaction(|tx| {
            tx.execute(sql, &[&id])
                .await
                .map_err(|e| to_error(e, Some(id.to_string())))?;
            Ok(())
        })
        .await
    }
}
