use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AboutMeView {
    pub id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<String>,
    pub picture: Option<String>,
}

impl AboutMeView {
    pub fn new(
        id: Option<i32>,
        first_name: String,
        last_name: String,
        description: Option<String>,
        picture: Option<String>,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            description,
            picture,
        }
    }
    pub fn id(&self) -> Option<&i32> {
        self.id.as_ref()
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}
