use crate::{app_error::Error, config::db::init_db, domain::about_me::AboutMe};

pub async fn update_about_me(id: i64, about: &AboutMe) -> Result<AboutMe, Error> {
    let db = init_db().await?;
    let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3, photo = $2 WHERE id = $4";
    let query = sqlx::query_as::<_, AboutMe>(&sql)
        .bind(id)
        .bind(about.first_name())
        .bind(about.last_name())
        .bind(about.description())
        .bind(about.picture());
    let about_me = query.fetch_one(&db).await?;
    Ok(about_me)
}
