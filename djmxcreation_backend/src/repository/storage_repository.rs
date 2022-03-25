use bytes::Buf;
use futures::Stream;
use tokio_stream::{self as stream, StreamExt};

use crate::{app_error::Error, config::minio::init_minio};

pub async fn upload_file(file_name: &str, file: std::vec::Vec<u8>) -> Result<(), Error> {
    let bucket = init_minio().await?;
    let mut stream = stream::iter(file);
    stream_ref.read()
    let status_code = bucket.put_object_stream(&mut stream_ref.read(file), "/path").await?;
    Ok(())
}

pub async fn get_object_url(file_name: &str) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_object(file_name: &str) -> Result<(), Error> {
    Ok(())
}
