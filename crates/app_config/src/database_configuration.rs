#[derive(Default, Clone)]
pub struct DatabaseConfiguration {
    pub url: String,
    pub pg_app_max_con: u32,
}

impl DatabaseConfiguration {
    pub fn new(url: String, pg_app_max_con: u32) -> Self {
        Self {
            url,
            pg_app_max_con,
        }
    }
}
