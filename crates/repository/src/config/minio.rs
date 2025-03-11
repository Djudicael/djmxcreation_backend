use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use s3::Bucket;
use s3::Region;
use s3::creds::Credentials;
use tracing::info;

pub type StorageClient = Box<Bucket>;

pub fn get_aws_client(config: StorageConfiguration) -> Result<StorageClient, Error> {
    let region = Region::Custom {
        region: config.region,
        endpoint: config.endpoint,
    };

    let credentials = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    )
    .expect("Should create credentials");

    let bucket = Bucket::new("portfolio", region, credentials).expect("Should create bucket");
    Ok(bucket)
}

// pub async fn create_bucket(bucket_name: &str, client: StorageClient) -> Result<(), Error> {
//     let credentials = client.credentials().await.expect("Should get credentials");
//     let new_bucket = Bucket::new(
//         bucket_name, // Use the provided bucket name
//         client.region().clone(),
//         credentials,
//     )
//     .expect("impossible to get the bucket");
//     // First check if bucket exists
//     // Check if bucket exists by trying to get its location
//     match new_bucket.get_location().await {
//         Ok(_) => {
//             info!("Bucket {bucket_name} already exists");
//             Ok(())
//         }
//         Err(_) => {
//             // Create the bucket by putting an empty object
//             new_bucket
//                 .put_object("/", &[])
//                 .await
//                 .map_err(|_| Error::BucketCreation)?;
//             info!("Bucket {bucket_name} created successfully");
//             Ok(())
//         }
//     }
// }
