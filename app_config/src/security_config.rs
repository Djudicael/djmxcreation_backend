#[derive(Default, Clone, Debug)]
pub struct SecurityConfig {
    pub username: String,
    pub password: String,
}

impl SecurityConfig {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}
