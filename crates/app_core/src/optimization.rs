use app_error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSource {
    pub url: String,
    pub width: i32,
    pub height: i32,
    pub mime_type: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOptimization {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub sources: Vec<ImageSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

pub type DynMediaProcessing = Arc<dyn IMediaProcessing + Send + Sync>;
#[async_trait]
pub trait IMediaProcessing {
    async fn images(&self, media: &[Uuid]) -> Result<HashMap<Uuid, ImageOptimization>, Error>;
    async fn cleanup(&self, limit: usize) -> Result<usize, Error>;
    async fn wake(&self) -> Result<bool, Error>;
}

pub fn enabled() -> bool {
    std::env::var("MEDIA_OPTIMIZATION_ENABLED").is_ok_and(|value| value == "true" || value == "1")
}

pub fn internal_authorized(value: Option<&str>) -> bool {
    let Ok(expected) = std::env::var("MEDIA_PROCESSING_TOKEN") else {
        return false;
    };
    if expected.len() < 32 {
        return false;
    }
    let supplied = value.unwrap_or_default().as_bytes();
    supplied.len() == expected.len()
        && supplied
            .iter()
            .zip(expected.as_bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}
