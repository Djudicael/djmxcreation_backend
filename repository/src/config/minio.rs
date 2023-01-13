use app_config::storage_configuration::StorageConfiguration;
use aws_sdk_s3::{config, Client, Credentials, Endpoint, Region};

use http::Uri;

use app_error::Error;
// "us-west-0"
pub fn get_aws_client(config: StorageConfiguration) -> Result<Client, Error> {
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
        .endpoint_resolver(Endpoint::immutable(config.endpoint.parse::<Uri>().unwrap()))
        .region(region)
        .credentials_provider(cred);
    let conf = conf_builder.build();
    // build aws client
    let client = Client::from_conf(conf);

    Ok(client)
}
