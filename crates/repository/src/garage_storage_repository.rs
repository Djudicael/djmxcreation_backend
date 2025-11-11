use app_core::storage::object_storage_repository::IObjectStorage;
use app_error::Error;
use async_trait::async_trait;
use chrono::Utc;
use hmac::Hmac;
use hmac::Mac;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::config::garage_client::GarageClient;

pub struct GarageStorageRepository {
    garage_client: GarageClient,
}

impl GarageStorageRepository {
    pub fn new(garage_client: GarageClient) -> Self {
        Self { garage_client }
    }
}

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize)]
struct CreateBucketPayload<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<&'a str>,
}

#[async_trait]
impl IObjectStorage for GarageStorageRepository {
    // Create bucket using admin API (preferred). If admin API isn't available/doesn't succeed,
    // we treat it as an error (mapping to Error::BucketCreation).
    async fn create_bucket(&self, name: &str, owner: Option<&str>) -> Result<(), Error> {
        // Use admin endpoint (trim trailing slash) and admin API path
        let admin_base = self.garage_client.admin_endpoint.trim_end_matches('/');
        let url = format!("{}/v1/buckets", admin_base);

        let payload = CreateBucketPayload { name, owner };

        let mut req = self.garage_client.client.put(&url).json(&payload);

        // Attach admin token if provided
        if let Some(token) = &self.garage_client.admin_token {
            req = req.bearer_auth(token);
        }

        let resp = req.send().await.map_err(|e| {
            eprintln!("create_bucket request error: {:?}", e);
            eprintln!(
                "stack trace: {:?}",
                std::backtrace::Backtrace::force_capture()
            );
            Error::BucketCreation
        })?;

        if !resp.status().is_success() {
            eprintln!("create_bucket failed: {} (status: {})", url, resp.status());
            return Err(Error::BucketCreation);
        }

        Ok(())
    }

    // Delete an object from the S3-compatible (s3_endpoint).
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), Error> {
        let base = self.garage_client.s3_endpoint.trim_end_matches('/');
        let url = format!("{}/{}/{}", base, bucket, key);

        let resp = self
            .garage_client
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("delete_object request error: {:?}", e);
                eprintln!(
                    "stack trace: {:?}",
                    std::backtrace::Backtrace::force_capture()
                );
                Error::StorageDeleteObject
            })?;

        if !resp.status().is_success() {
            eprintln!("delete_object failed: {} (status: {})", url, resp.status());
            return Err(Error::StorageDeleteObject);
        }

        Ok(())
    }

    // Upload bytes to S3-compatible endpoint.
    async fn upload_stream(&self, bucket: &str, key: &str, data: &[u8]) -> Result<(), Error> {
        let base = self.garage_client.s3_endpoint.trim_end_matches('/');
        let url = format!("{}/{}/{}", base, bucket, key);

        let resp = self
            .garage_client
            .client
            .put(&url)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| {
                eprintln!("upload_stream request error: {:?}", e);
                eprintln!(
                    "stack trace: {:?}",
                    std::backtrace::Backtrace::force_capture()
                );
                Error::StorageUpload
            })?;

        if !resp.status().is_success() {
            eprintln!("upload_stream failed: {} (status: {})", url, resp.status());
            return Err(Error::StorageUpload);
        }

        Ok(())
    }

    // Return a public URL (prefer the configured public_endpoint; fallback to s3 endpoint).
    // Verify the object exists by performing a HEAD on the S3 endpoint before returning.
    async fn get_public_url(&self, bucket: &str, key: &str) -> Result<String, Error> {
        let public_base = self
            .garage_client
            .public_endpoint
            .as_deref()
            .map(|s| s.trim_end_matches('/'))
            .unwrap_or_else(|| self.garage_client.s3_endpoint.trim_end_matches('/'));

        let public_url = format!("{}/{}/{}", public_base, bucket, key);

        // Check existence against the S3 endpoint (canonical storage).
        let s3_base = self.garage_client.s3_endpoint.trim_end_matches('/');
        let head_url = format!("{}/{}/{}", s3_base, bucket, key);

        let head_resp = self
            .garage_client
            .client
            .head(&head_url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("get_public_url HEAD request error: {:?}", e);
                eprintln!(
                    "stack trace: {:?}",
                    std::backtrace::Backtrace::force_capture()
                );
                Error::StorageGetObjectUrl
            })?;

        if !head_resp.status().is_success() {
            eprintln!(
                "get_public_url: object not found or inaccessible: {} (status: {})",
                head_url,
                head_resp.status()
            );
            return Err(Error::StorageGetObjectUrl);
        }

        Ok(public_url)
    }

    // Generate a signed URL using an AWS4-like signature compatible with GarageHQ's S3 interface.
    // Perform an existence check first (HEAD).
    async fn get_signed_url(
        &self,
        bucket: &str,
        key: &str,
        expires_secs: u32,
    ) -> Result<String, Error> {
        // Existence check against S3 endpoint.
        let s3_base = self.garage_client.s3_endpoint.trim_end_matches('/');
        let object_url = format!("{}/{}/{}", s3_base, bucket, key);

        let head_resp = self
            .garage_client
            .client
            .head(&object_url)
            .send()
            .await
            .map_err(|e| {
                eprintln!("get_signed_url HEAD request error: {:?}", e);
                eprintln!(
                    "stack trace: {:?}",
                    std::backtrace::Backtrace::force_capture()
                );
                Error::StorageGetObjectUrl
            })?;

        if !head_resp.status().is_success() {
            eprintln!(
                "get_signed_url: object not found or inaccessible: {} (status: {})",
                object_url,
                head_resp.status()
            );
            return Err(Error::StorageGetObjectUrl);
        }

        // Build AWS4-like signed URL
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        // Host (without scheme) used for signing
        let host = s3_base.replace("http://", "").replace("https://", "");

        let canonical_uri = format!("/{}/{}", bucket, key);
        let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, "garage", "s3");

        // Avoid creating a temporary format! value that's borrowed; take an owned copy of the access key
        let credential_input = self.garage_client.access_key.to_owned();
        let credential = urlencoding::encode(&credential_input);

        // Build query portion (note we URL-encode credential scope via %2F in credential param)
        let query = format!(
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders=host",
            credential, credential_scope, amz_date, expires_secs
        );

        // Canonical request (note the query is the entire query string from above)
        let canonical_request = format!(
            "GET\n{}\n{}\nhost:{}\n\nhost\nUNSIGNED-PAYLOAD",
            canonical_uri, query, host
        );
        let hashed_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date, credential_scope, hashed_request
        );

        // Derive signing key: AWS4(secret) -> date -> region -> service -> signing
        let mut k_date =
            HmacSha256::new_from_slice(format!("AWS4{}", self.garage_client.secret_key).as_bytes())
                .unwrap();
        k_date.update(date_stamp.as_bytes());
        let k_date = k_date.finalize().into_bytes();

        let mut k_region = HmacSha256::new_from_slice(&k_date).unwrap();
        k_region.update(b"garage");
        let k_region = k_region.finalize().into_bytes();

        let mut k_service = HmacSha256::new_from_slice(&k_region).unwrap();
        k_service.update(b"s3");
        let k_service = k_service.finalize().into_bytes();

        let mut k_signing = HmacSha256::new_from_slice(&k_service).unwrap();
        k_signing.update(b"aws4_request");
        let k_signing = k_signing.finalize().into_bytes();

        let mut mac = HmacSha256::new_from_slice(&k_signing).unwrap();
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // Build final signed URL against the S3 endpoint. Note: if you want signed URLs to be served
        // through a public CDN (public_endpoint), the host used to sign must match the host that will
        // receive the request (so you'd need to sign for that host instead).
        let signed_url = format!(
            "{}{}?{}&X-Amz-Signature={}",
            s3_base, canonical_uri, query, signature
        );

        Ok(signed_url)
    }
}
