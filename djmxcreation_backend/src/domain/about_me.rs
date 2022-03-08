use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutMe {
    pub first_name: String,
    pub last_name: String,
    pub description: Option<String>,
    pub picture: Option<String>,
}

impl AboutMe {
    pub fn new(
        first_name: String,
        last_name: String,
        description: Option<String>,
        picture: Option<String>,
    ) -> Self {
        Self {
            first_name,
            last_name,
            description,
            picture,
        }
    }
}
