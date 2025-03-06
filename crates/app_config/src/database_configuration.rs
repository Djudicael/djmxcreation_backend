#[derive(Default)]
pub struct DatabaseConfiguration {
    pub pg_host: String,
    pub pg_db: String,
    pub pg_user: String,
    pub pg_password: String,
    pub pg_app_max_con: u32,
    pub pg_port: u16,
}

impl DatabaseConfiguration {
    pub fn new(
        pg_host: String,
        pg_db: String,
        pg_user: String,
        pg_password: String,
        pg_app_max_con: u32,
        pg_port: u16,
    ) -> Self {
        Self {
            pg_host,
            pg_db,
            pg_user,
            pg_password,
            pg_app_max_con,
            pg_port,
        }
    }
}
