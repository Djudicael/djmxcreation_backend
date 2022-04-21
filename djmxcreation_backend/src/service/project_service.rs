use chrono::{Utc, DateTime};

use crate::{app_error::Error, domain::metadata::Metadata};

pub async fn create_project(metadate: &Metadata) -> Result<(), Error> {
    let now_utc: DateTime<Utc> = Utc::now();
    Ok(())
}
