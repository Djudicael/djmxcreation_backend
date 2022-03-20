use bytes::BufMut;
use futures::TryStreamExt;
use warp::multipart::Part;

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

pub async fn add_profile_picture(id: i32, file: Part) -> Result<AboutMe, Error> {
    let content_type = file.content_type();
            let file_ending;
            // Verify the type of file sent
            match content_type {
                Some(file_type) => match file_type {
                    "image/jpg" => {
                        file_ending = "jpg";
                    }
                    "image/jpeg" => {
                        file_ending = "jpeg";
                    }
                    "image/png" => {
                        file_ending = "png";
                    }
                    // v => {
                    //     eprintln!("invalid file type found: {}", v);
                    //     return Err(warp::reject::reject());
                    // }
                },
                None => {
                    eprintln!("file type could not be determined");
                    // return Err(warp::reject::reject());
                }
            }
    let value = file
        .stream()
        .try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        })
        .await;

        

        //TODO fix
    let file_name =file.filename().map(|name| name.to_string()).map_or()
    (default, f) format!("./files/{}.{}", Uuid::new_v4().to_string(), file_ending);
    //TODO try to validate id
    let result = update_about_me(id, about).await?;
    Ok(result)
}
