#[derive(Default)]
pub struct StorageConfiguration {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

impl StorageConfiguration {
    pub fn new(endpoint: String, access_key: String, secret_key: String, region: String) -> Self {
        Self {
            endpoint,
            access_key,
            secret_key,
            region,
        }
    }
}
