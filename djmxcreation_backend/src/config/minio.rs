
use aws_sdk_s3::Client;
use dotenv::dotenv;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region, request_trait::Request};
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

fn get_aws_client(region: &str) -> Result<Client> {
	// get the id/secret from env
    dotenv().unwrap();
    let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap();
    let minio_access_key = env::var("MINIO_ACCESS_KEY").unwrap();
    let minio_secret_key = env::var("MINIO_SECRET_KEY").unwrap();

	// build the aws cred
	let cred = Credentials::new(Some(&minio_access_key.as_str()), Some(&minio_secret_key.as_str()), None, None, "loaded-from-custom-env");

	// build the aws client
	let region = Region::new(region.to_string());
	let conf_builder = config::Builder::new().region(region).credentials_provider(cred).url();
	let mut conf = conf_builder.build();

    // conf

	// build aws client
	let client = Client::from_conf(conf);
	Ok(client)
}
