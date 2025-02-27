use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentView {
    id: Option<Uuid>,
    mime_type: Option<String>,
    url: Option<String>,
}

impl ContentView {
    pub fn new(id: Option<Uuid>, mime_type: Option<String>, url: Option<String>) -> Self {
        Self { id, mime_type, url }
    }
}
