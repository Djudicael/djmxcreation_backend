use std::collections::HashMap;

use testcontainers::{clients::Cli, core::WaitFor, Image, Container};

const NAME: &str = "postgres";
const TAG: &str = "11-alpine";

pub struct PostgresSQL {
    env_vars: HashMap<String, String>,
}

impl PostgresSQL {
    pub fn new(env_vars: HashMap<String, String>) -> Self {
        Self { env_vars }
    }
}
impl Default for PostgresSQL {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("POSTGRES_DB".to_owned(), "postgres".to_owned());
        env_vars.insert("POSTGRES_HOST_AUTH_METHOD".into(), "trust".into());

        Self { env_vars }
    }
}

impl Image for PostgresSQL {
    type Args = ();

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}

pub fn init_postgresql(pg_db: &str, pg_user: &str, pg_password: &str) -> Container<PostgresSQL> {
    let mut env_vars = HashMap::new();
    env_vars.insert("POSTGRES_DB".to_owned(), pg_db.to_owned());
    env_vars.insert("POSTGRES_PASSWORD".into(), pg_password.into());
    env_vars.insert("POSTGRES_USER".into(), pg_user.into());

    let image = PostgresSQL::new(env_vars);

    let podman = Cli::podman();

    podman.run(image);
}
