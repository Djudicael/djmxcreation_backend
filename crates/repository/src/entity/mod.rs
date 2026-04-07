pub mod about_me;
pub mod contact;
pub mod project;
pub mod project_content;
pub mod project_with_thumbnail;
pub mod spotlight;

use app_core::dto::{content_dto::ContentDto, metadata_dto::MetadataDto};
use serde_json::Value;

/// Deserialize a JSON `Value` into a `ContentDto`, returning `None` on failure.
pub fn value_to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}

/// Deserialize a JSON `Value` into a `MetadataDto`, returning `None` on failure.
pub fn value_to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}
