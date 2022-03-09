use crate::{app_error::Error, domain::about_me::AboutMe, config::db::init_db};

pub async fn update_about_me(about: &AboutMe) -> Result<AboutMe, Error> {
    let db = init_db().await?;

    unimplemented!()
}
