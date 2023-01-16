use app_config::storage_configuration::StorageConfiguration;
use aws_sdk_s3::{config, types::SdkError, Client, Credentials, Endpoint, Region};

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

// pub async fn create_bucket_old(bucket_name: &str, client: StorageClient) -> Result<(), Error> {
//     let info = client.head_bucket().bucket(bucket_name).send().await;

//     match info {
//         Ok(_) => println!("Bucket {} exists!", bucket_name),
//         Err(err) => {
//             if let Some(status) = err.ki {
//                 if status == 404 {
//                     client
//                         .create_bucket()
//                         .bucket(bucket_name)
//                         .send()
//                         .await
//                         .unwrap();
//                     println!("Bucket {} created!", bucket_name);
//                 } else {
//                     println!("Error checking if bucket {} exists: {:?}", bucket_name, err);
//                     return Err(err);
//                 }
//             } else {
//                 println!("Error checking if bucket {} exists: {:?}", bucket_name, err);
//                 return Err(err);
//             }
//         }
//     }

//     Ok(())
// }

pub async fn create_bucket(bucket_name: &str, client: StorageClient) -> Result<(), Error> {
    client
        .create_bucket()
        .bucket(bucket_name)
        .send()
        .await
        .expect("Error cannot crate the bucket {bucket_name} parsing");

    Ok(())
}
