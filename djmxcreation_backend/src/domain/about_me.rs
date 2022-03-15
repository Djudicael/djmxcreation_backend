use crate::domain::content::Content;
use serde::{Deserialize, Serialize};
#[derive(sqlx::FromRow, Serialize, Deserialize, Default, Debug, Clone)]
pub struct AboutMe {
    id: Option<i64>,
    first_name: String,
    last_name: String,
    description: Option<String>,
    picture: Option<String>,
}

impl AboutMe {
    pub fn new(
        id: Option<i64>,
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

    pub fn id(&self) -> Option<&i64> {
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

    pub fn picture(&self) -> Option<&String> {
        self.picture.as_ref()
    }
}
