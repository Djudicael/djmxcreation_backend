use crate::{app_error::Error, config::db::init_db, domain::about_me::AboutMe};

// https://jmoiron.github.io/sqlx/

pub async fn update_about_me(id: i32, about: &AboutMe) -> Result<AboutMe, Error> {
    let db = init_db().await?;
    let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3 WHERE id = $4 RETURNING *";
    // let sql = "UPDATE about SET first_name = $1, last_name = $2, description = $3, photo = $4 WHERE id = $5 RETURNING *";
    let query = sqlx::query_as::<_, AboutMe>(&sql)
        .bind(about.first_name())
        .bind(about.last_name())
        .bind(about.description())
        // .bind(about.photo())
        .bind(id);
    let about_me = query.fetch_one(&db).await?;
    Ok(about_me)
}

pub async fn get_about_me() -> Result<AboutMe, Error> {
    let db = init_db().await?;
    let sql = "SELECT * FROM about FETCH FIRST ROW ONLY";
    let query = sqlx::query_as::<_, AboutMe>(&sql);
    let about_me = query.fetch_one(&db).await?;
    Ok(about_me)
}

// export const updatePhoto = async (idProjectId, photo) => {
//     const client = await pool.connect();
//     try {
//         await client.query('BEGIN');
//         const queryText = `UPDATE sylwia_portfolio.${TABLE_ABOUT}
//         SET photo = $1
//         WHERE id = $2;`;
//         const { rows } = await client.query(queryText, [photo, idProjectId]);
//         await client.query('COMMIT')
//         return rows[0];
//     } catch (e) {
//         console.log(e);
//         await client.query('ROLLBACK')
//         throw e
//     } finally {
//         client.release()
//     }
// };
