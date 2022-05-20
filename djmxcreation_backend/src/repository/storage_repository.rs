use aws_sdk_s3::types::ByteStream;

use crate::{
    app_error::Error,
    config::minio::{get_aws_client, get_s3_client},
};

pub async fn upload_file(
    bucket_name: &str,
    file_name: &str,
    file: &std::vec::Vec<u8>,
) -> Result<(), Error> {
    let client = get_aws_client("us-west-0")?;

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

pub async fn get_object_url(bucket_name: &str, file_name: &str) -> Result<String, Error> {
    // client
    let client = get_s3_client(bucket_name, "us-west-0")?;
    let url = client.presign_get(file_name, 8640, None).unwrap();
    Ok(url)
}

pub async fn remove_object(bucket_name: &str, file_name: &str) -> Result<(), Error> {
    let client = get_aws_client("us-west-0")?;
    client
        .delete_object()
        .bucket(bucket_name)
        .key(file_name)
        .send()
        .await?;
    Ok(())
}
