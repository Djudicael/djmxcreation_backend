#[derive(Default, Clone)]
pub struct StorageConfiguration {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub admin_endpoint: String,
    pub admin_token: String,
}

impl StorageConfiguration {
    pub fn new(
        endpoint: String,
        access_key: String,
        secret_key: String,
        region: String,
        admin_endpoint: String,
        admin_token: String,
    ) -> Self {
        Self {
            endpoint,
            access_key,
            secret_key,
            region,
            admin_endpoint,
            admin_token,
        }
    }
}
