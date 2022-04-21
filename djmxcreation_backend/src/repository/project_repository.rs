use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::types::Json;

use crate::{app_error::Error, config::db::init_db, domain::{metadata::Metadata, project_entity::ProjectEntity}};

pub async fn create(metadata: &Metadata) -> Result<ProjectEntity, Error> {
    let db = init_db().await?;
    let metadataJson = Json(json!(metadata));
    let now_utc: DateTime<Utc> = Utc::now();
    let sql = "INSERT INTO sylwia_portfolio.project(metadata, created_on, visible) VALUES($1, $2, $3) RETURNING *";
    let query = sqlx::query_as::<_, ProjectEntity>(&sql)
        .bind(metadataJson)
        .bind(now_utc)
        .bind(false);

    let id = query.fetch_one(&db).await?;
    Ok(id)
}
