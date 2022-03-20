
use dotenv::dotenv;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use std::env;

use crate::app_error::Error;
pub async fn init_minio() -> Result<Bucket, Error> {
    dotenv().unwrap();
    let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap();
    let minio_access_key = env::var("MINIO_ACCESS_KEY").unwrap();
    let minio_secret_key = env::var("MINIO_SECRET_KEY").unwrap();
    // 1 instantiate the bucket client
    let bucket = Bucket::new_with_path_style(
        "portfolio",
        Region::Custom {
            region: "".to_owned(),
            endpoint: minio_endpoint,
        },
        Credentials {
            access_key: Some(minio_access_key),
            secret_key: Some(minio_secret_key),
            security_token: None,
            session_token: None,
        },
    ).unwrap();
    // 2 create bucket if doesnt  not exist
    let (_, code) = bucket.head_object("/").await.unwrap();
    if code == 404 {
        let create_result = Bucket::create_with_path_style(
            bucket.name.as_str(),
            bucket.region.clone(),
            bucket.credentials.clone(),
            BucketConfiguration::default(),
        )
        .await.unwrap();
        println!(
            "==== Bucket created \n{} - {} - {}",
            bucket.name, create_result.response_code, create_result.response_text
        );
    }

    Ok(bucket)
}
