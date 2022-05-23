use aws_sdk_s3::{config, Client, Credentials, Endpoint, Region};
use dotenv::dotenv;
use s3::Bucket;

// use s3::{creds::Credentials, request_trait::Request, Bucket, BucketConfiguration, Region};
use std::{env, str::FromStr};
use warp::http::Uri;

use crate::app_error::Error;

pub fn get_aws_client(region: &str) -> Result<Client, Error> {
    // get the id/secret from env
    dotenv().unwrap();
    let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap();
    let minio_access_key = env::var("MINIO_ACCESS_KEY").unwrap();
    let minio_secret_key = env::var("MINIO_SECRET_KEY").unwrap();

    // build the aws cred
    let cred = Credentials::new(
        minio_access_key.as_str(),
        minio_secret_key.as_str(),
        None,
        None,
        "",
        // "loaded-from-custom-env",
    );

    let region = Region::new("us-west-0");

    let conf_builder = config::Builder::new()
        .endpoint_resolver(Endpoint::immutable(minio_endpoint.parse::<Uri>().unwrap()))
        .region(region)
        .credentials_provider(cred);
    let conf = conf_builder.build();
    // build aws client
    let client = Client::from_conf(conf);

    Ok(client)
}

pub fn get_s3_client(bucket_name: &str, region: &str) -> Result<Bucket, Error> {
    dotenv().unwrap();
    let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap();
    let minio_access_key = env::var("MINIO_ACCESS_KEY").unwrap();
    let minio_secret_key = env::var("MINIO_SECRET_KEY").unwrap();

    let bucket = Bucket::new(
        bucket_name,
        s3::Region::Custom {
            region: region.to_owned(),
            endpoint: minio_endpoint,
        },
        s3::creds::Credentials {
            access_key: Some(minio_access_key),
            secret_key: Some(minio_secret_key),
            security_token: None,
            session_token: None,
        },
    )
    .unwrap()
    .with_path_style();

    Ok(bucket)
}
