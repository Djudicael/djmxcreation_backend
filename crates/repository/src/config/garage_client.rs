use reqwest::Client;

pub struct GarageClient {
    /// S3-compatible endpoint (upload/download)
    pub s3_endpoint: String,

    /// Admin API endpoint (bucket/key management)
    pub admin_endpoint: String,

    /// Public endpoint (e.g. CDN / public host) - optional
    pub public_endpoint: Option<String>,

    /// Access credentials for S3 operations
    pub access_key: String,
    pub secret_key: String,

    /// Admin token (Bearer) for admin API (optional; required for bucket management if enabled)
    pub admin_token: Option<String>,

    /// Internal HTTP client
    pub client: Client,
}

impl GarageClient {
    pub fn new(
        s3_endpoint: String,
        admin_endpoint: String,
        public_endpoint: Option<String>,
        access_key: String,
        secret_key: String,
        admin_token: Option<String>,
    ) -> Self {
        Self {
            s3_endpoint: s3_endpoint.into(),
            admin_endpoint: admin_endpoint.into(),
            public_endpoint,
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            admin_token,
            client: Client::new(),
        }
    }
}
