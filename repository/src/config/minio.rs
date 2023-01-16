use app_config::storage_configuration::StorageConfiguration;
use aws_sdk_s3::{
    config, error::HeadBucketErrorKind, types::SdkError, Client, Credentials, Endpoint, Region,
};
use tracing::info;
use app_error::Error;

pub type StorageClient = Client;
// "us-west-0"
pub fn get_aws_client(config: StorageConfiguration) -> Result<StorageClient, Error> {
    // build the aws cred
    let cred = Credentials::new(
        config.access_key.as_str(),
        config.secret_key.as_str(),
        None,
        None,
        "",
        // "loaded-from-custom-env",
    );

    let region = Region::new(config.region);

    let conf_builder = config::Builder::new()
        .endpoint_resolver(Endpoint::immutable(config.endpoint).expect("Error endpoint parsing"))
        .region(region)
        .credentials_provider(cred);
    let conf = conf_builder.build();
    // build aws client
    let client = Client::from_conf(conf);

    Ok(client)
}

pub async fn create_bucket(bucket_name: &str, client: StorageClient) -> Result<(), Error> {
    let metadata = client.head_bucket().bucket(bucket_name).send().await;

    // Create bucket when it doesn't exist
    match metadata {
        Ok(_) => println!("Bucket {bucket_name} exists!"),
        Err(err) => match err {
            SdkError::ServiceError(sdk_err) => match sdk_err.err().kind {
                HeadBucketErrorKind::NotFound(_) => {
                    client
                        .create_bucket()
                        .bucket(bucket_name)
                        .send()
                        .await
                        .expect("Error cannot create the bucket {bucket_name} parsing");
                    info!("Bucket {} created!", bucket_name);
                }
                _ => {
                    return Err(Error::BucketCreation);
                }
            },
            _ => {
                info!("Error checking if bucket {} exists: {:?}", bucket_name, err);
                return Err(Error::BucketCreation);
            }
        },
    }

    Ok(())
}

// pub async fn create_bucket(bucket_name: &str, client: StorageClient) -> Result<(), Error> {
//     client
//         .create_bucket()
//         .bucket(bucket_name)
//         .send()
//         .await
//         .expect("Error cannot create the bucket {bucket_name} parsing");

//     Ok(())
// }
