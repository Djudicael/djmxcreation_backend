use crate::{
    app_error::Error,
    domain::about_me::AboutMe,
    repository::about_me_repository::{get_about_me, update_about_me},
};

pub async fn about_me() -> Result<AboutMe, Error> {
    let about_me = get_about_me().await?;
    Ok(about_me)
}

pub async fn update_me(id: i32, about: &AboutMe) -> Result<AboutMe, Error> {
    //TODO try to validate id
    let result = update_about_me(id, about).await?;
    Ok(result)
}
