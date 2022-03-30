use aws_sdk_s3::types::ByteStream;
use bytes::Buf;
use futures::Stream;
use tokio_stream::{self as stream, StreamExt};

use crate::{app_error::Error, config::minio::get_aws_client};

pub async fn upload_file(bucket_name: &str,file_name: &str, file: &std::vec::Vec<u8>) -> Result<(), Error> {
  
    let client = get_aws_client("test")?;

    let body = ByteStream::from(file.clone());
    client
        .put_object()
        .bucket(bucket_name)
        .key(file_name)
        .body(body)
        .send()
        .await?;
    Ok(())
}

pub async fn get_object_url(file_name: &str) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_object(file_name: &str) -> Result<(), Error> {
    Ok(())
}
