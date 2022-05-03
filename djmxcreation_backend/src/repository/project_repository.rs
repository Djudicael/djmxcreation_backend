use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::types::Json;

use crate::{
    app_error::Error,
    config::db::init_db,
    domain::{
        content::Content, metadata::Metadata, project_content_entity::ProjectContentEntity,
        project_entity::ProjectEntity,
    },
};

pub async fn create(metadata: &Metadata) -> Result<ProjectEntity, Error> {
    let db = init_db().await?;
    let metadata_json = Json(json!(metadata));
    let now_utc: DateTime<Utc> = Utc::now();
    let sql = "INSERT INTO project(metadata, created_on, visible) VALUES($1, $2, $3) RETURNING *";
    let query = sqlx::query_as::<_, ProjectEntity>(&sql)
        .bind(metadata_json)
        .bind(now_utc)
        .bind(false);

    let project = query.fetch_one(&db).await?;
    Ok(project)
}

pub async fn add_project_content(
    project_id: i32,
    content: &Content,
) -> Result<ProjectContentEntity, Error> {
    let db = init_db().await?;
    let content_json = Json(json!(content));
    let now_utc: DateTime<Utc> = Utc::now();
    let sql = "INSERT INTO project_content(project_id, content, created_on) VALUES($1, $2, $3) RETURNING *";
    let query = sqlx::query_as::<_, ProjectContentEntity>(&sql)
        .bind(project_id)
        .bind(content_json)
        .bind(now_utc);
    let content_entity = query.fetch_one(&db).await?;
    Ok(content_entity)
}

pub async fn update_description(project_id: i32, description: &Value) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE project SET description = $1 WHERE id = $2 ")
        .bind(Json(description))
        .bind(project_id)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn update_metadata(project_id: i32, metadata: &Metadata) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE project SET metadata = $1 WHERE id = $2 ")
        .bind(Json(metadata))
        .bind(project_id)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn update_visibility(project_id: i32, is_visible: bool) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE project SET visible = $1 WHERE id = $2 ")
        .bind(is_visible)
        .bind(project_id)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_projects() -> Result<Vec<ProjectEntity>, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM project";
    let query = sqlx::query_as::<_, ProjectEntity>(&sql);
    let projects = query.fetch_all(&db).await?;
    Ok(projects)
}

pub async fn get_project_by_id(id: i32) -> Result<ProjectEntity, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM project where id= $1";
    let query = sqlx::query_as::<_, ProjectEntity>(&sql).bind(id);
    let project = query.fetch_one(&db).await?;
    Ok(project)
}

pub async fn get_projects_contents(project_id: i32) -> Result<Vec<ProjectContentEntity>, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM project_content where project_id = $1";
    let query = sqlx::query_as::<_, ProjectContentEntity>(&sql).bind(project_id);
    let contents = query.fetch_all(&db).await?;
    Ok(contents)
}

pub async fn get_projects_content_by_id(
    project_id: i32,
    id: i32,
) -> Result<ProjectContentEntity, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM project_content where project_id = $1 and id= $2";
    let query = sqlx::query_as::<_, ProjectContentEntity>(&sql)
        .bind(project_id)
        .bind(id);
    let content = query.fetch_one(&db).await?;
    Ok(content)
}

pub async fn delete_project_by_id(project_id: i32) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;
    sqlx::query("DELETE project WHERE id = $1 ")
        .bind(project_id)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_project_content_by_id(project_id: i32, id: i32) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;
    sqlx::query("DELETE project_content WHERE id = $1 and project_id=$2 ")
        .bind(id)
        .bind(project_id)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
