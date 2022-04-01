use bytes::BufMut;
use futures::TryStreamExt;
use tokio_stream::{self as stream, StreamExt};
use uuid::Uuid;
use warp::multipart::Part;

use crate::{
    app_error::Error,
    domain::about_me::AboutMe,
    repository::{
        about_me_repository::{get_about_me, update_about_me},
        storage_repository::upload_file,
    },
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

pub async fn add_profile_picture(
    id: i32,
    file_name: String,
    file: &std::vec::Vec<u8>,
) -> Result<AboutMe, Error> {
    upload_file(&"portfolio/about", &file_name.as_str(), file).await?;

    //TODO try to validate id
    // let result = update_about_me(id, about).await?;
    Ok(AboutMe::default())
}
