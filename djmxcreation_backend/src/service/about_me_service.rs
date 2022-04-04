use std::ptr::NonNull;

use crate::{
    app_error::Error,
    domain::about_me::AboutMe,
    domain::content::Content,
    domain::me::Me,
    repository::{
        about_me_repository::{get_about_me, get_about_me_by_id, update_about_me, update_photo},
        storage_repository::{get_object_url, remove_object, upload_file},
    },
};

pub async fn about_me() -> Result<Me, Error> {
    let about_me = get_about_me().await?;

    let content = about_me
        .photo()
        .map(|photo| &photo.0)
        .map(|photo| to_content(photo));

    let url = match content {
        Some(photo) => {
            let url = get_object_url(photo.bucket_name(), photo.file_name()).await?;
            Some(url)
        }
        None => None,
    };

    let me = Me::new(
        about_me.id().map(|id| *id),
        about_me.first_name().to_string(),
        about_me.last_name().to_string(),
        about_me
            .description()
            .map(|description| description.clone()),
        about_me.photo().map(|photo| photo.clone()),
        url,
    );
    Ok(me)
}

pub async fn update_me(id: i32, about: &AboutMe) -> Result<Me, Error> {
    let _ = get_about_me_by_id(id).await?;
    let result = update_about_me(id, about).await?;
    let content = result
        .photo()
        .map(|photo| &photo.0)
        .map(|photo| to_content(photo));
    let url = match content {
        Some(photo) => {
            let url = get_object_url(photo.bucket_name(), photo.file_name()).await?;
            Some(url)
        }
        None => None,
    };
    let me = Me::new(
        result.id().map(|id| *id),
        result.first_name().to_string(),
        result.last_name().to_string(),
        result.description().map(|description| description.clone()),
        result.photo().map(|photo| photo.clone()),
        url,
    );
    Ok(me)
}

fn to_content(value: &serde_json::Value) -> Content {
    serde_json::from_value(value.clone()).unwrap()
}

pub async fn add_profile_picture(
    id: i32,
    file_name: String,
    file: &std::vec::Vec<u8>,
) -> Result<(), Error> {
    let me = get_about_me_by_id(id).await?;
    let key = format!("{}/{}", "about", file_name);
    let previous_content = me
        .photo()
        .map(|photo| &photo.0)
        .map(|photo| to_content(photo));
    let bucket = "portfolio";
    let content = Content::new(None, bucket.to_owned(), key.clone(), None);
    upload_file(bucket, &key.as_str(), file).await?;
    update_photo(id, &content).await?;

    // delete previous image from bucket
    match previous_content {
        Some(content) => remove_object(content.bucket_name(), content.file_name()).await?,
        None => (),
    }
    Ok(())
}
