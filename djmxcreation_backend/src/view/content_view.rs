use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentView {
    id: Option<i32>,
    mime_type: Option<String>,
    url: Option<String>,
}

impl ContentView {
    pub fn new(id: Option<i32>, mime_type: Option<String>, url: Option<String>) -> Self {
        Self { id, mime_type, url }
    }
}
