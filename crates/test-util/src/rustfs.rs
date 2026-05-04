/// RustFS test constants.
pub const RUSTFS_REGION: &str = "us-east-1";
pub const RUSTFS_ACCESS_KEY: &str = "rustfsadmin";
pub const RUSTFS_SECRET_KEY: &str = "rustfsadmin";

/// Build an endpoint URL from a host port.
pub fn rustfs_endpoint(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}
