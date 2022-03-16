use crate::{
    app_error::Error, domain::about_me::AboutMe, repository::about_me_repository::get_about_me,
};

pub async fn about_me() -> Result<AboutMe, Error> {
    let about_me = get_about_me().await?;
    Ok(about_me)
}
