use std::error::Error;

use dotenv::dotenv;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use std::env;
pub async fn init_minio() -> Result<Bucket, Box<dyn Error>> {
    dotenv()?;
    let minio_endpoint = env::var("MINIO_ENDPOINT")?;
    let minio_access_key = env::var("MINIO_ACCESS_KEY")?;
    let minio_secret_key = env::var("MINIO_SECRET_KEY")?;
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
    )?;
    // 2 create bucket if doesnt  not exist
    let (_, code) = bucket.head_object("/").await?;
    if code == 404 {
        let create_result = Bucket::create_with_path_style(
            bucket.name.as_str(),
            bucket.region.clone(),
            bucket.credentials.clone(),
            BucketConfiguration::default(),
        )
        .await?;
        println!(
            "==== Bucket created \n{} - {} - {}",
            bucket.name, create_result.response_code, create_result.response_text
        );
    }

    Ok(bucket)
}
