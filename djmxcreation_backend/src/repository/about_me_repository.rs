use crate::{
    app_error::Error,
    config::db::init_db,
    domain::{about_me::AboutMe, content::Content},
};

use serde_json::json;
use sqlx::types::Json;

// https://jmoiron.github.io/sqlx/

pub async fn update_about_me(id: i32, about: &AboutMe) -> Result<AboutMe, Error> {
    let db = init_db().await?;
    let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3 WHERE id = $4 RETURNING *";
    // let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3, photo = $4 WHERE id = $5 RETURNING *";
    let query = sqlx::query_as::<_, AboutMe>(&sql)
        .bind(about.first_name())
        .bind(about.last_name())
        .bind(about.description())
        .bind(id);
    let about_me = query
        .fetch_one(&db)
        .await
        .map_err(|sqlx_error| match sqlx_error {
            sqlx::Error::RowNotFound => Error::EntityNotFound(id.to_string()),
            other => Error::SqlxError(other),
        })?;
    Ok(about_me)
}

pub async fn get_about_me() -> Result<AboutMe, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM about FETCH FIRST ROW ONLY";
    let query = sqlx::query_as::<_, AboutMe>(&sql);
    let about_me = query.fetch_one(&db).await?;
    Ok(about_me)
}

pub async fn get_about_me_by_id(id: i32) -> Result<AboutMe, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM about where id = $1 FETCH FIRST ROW ONLY";
    let query = sqlx::query_as::<_, AboutMe>(&sql).bind(id);
    let about_me = query
        .fetch_one(&db)
        .await
        .map_err(|sqlx_error| match sqlx_error {
            sqlx::Error::RowNotFound => Error::EntityNotFound(id.to_string()),
            other => Error::SqlxError(other),
        })?;
    Ok(about_me)
}

pub async fn update_photo(id: i32, content: &Content) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;

    let content_json = Json(json!(content));

    sqlx::query("UPDATE about SET photo = $1 WHERE id = $2 ")
        .bind(content_json)
        .bind(id)
        .execute(&mut tx)
        .await
        .map_err(|sqlx_error| match sqlx_error {
            sqlx::Error::RowNotFound => Error::EntityNotFound(id.to_string()),
            other => Error::SqlxError(other),
        })?;

    tx.commit().await?;

    Ok(())
}

pub async fn delete_about_me_photo(id: i32) -> Result<(), Error> {
    let db = init_db().await?;
    let mut tx = db.begin().await?;
    sqlx::query("UPDATE about SET photo = NULL WHERE id = $1")
        .bind(id)
        .execute(&mut tx)
        .await
        .map_err(|sqlx_error| match sqlx_error {
            sqlx::Error::RowNotFound => Error::EntityNotFound(id.to_string()),
            other => Error::SqlxError(other),
        })?;
    tx.commit().await?;
    Ok(())
}
