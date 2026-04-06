use std::{future::Future, pin::Pin};

use app_error::Error;
use deadpool_postgres::PoolError;
use tokio_postgres::Transaction;

use crate::{config::db::DatabasePool, error::to_error};

/// Run `f` inside a database transaction, committing on success and
/// rolling back (implicitly, on drop) on error.
pub async fn with_transaction<F, T>(pool: &DatabasePool, f: F) -> Result<T, Error>
where
    F: for<'a> FnOnce(
        &'a Transaction<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>,
{
    let mut client = pool.get_client().await.map_err(|e| to_error(e, None))?;

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
