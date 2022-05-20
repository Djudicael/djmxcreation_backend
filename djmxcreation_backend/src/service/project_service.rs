use chrono::{DateTime, Utc};

use crate::{app_error::Error, domain::metadata::Metadata, repository::project_repository::create};

pub async fn create_project(metadata: &Metadata) -> Result<(), Error> {
    let project = create(metadata).await?;
    Ok(())
}
