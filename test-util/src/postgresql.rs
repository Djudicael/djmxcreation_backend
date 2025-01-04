use std::{borrow::Cow, collections::HashMap};

use rustainers::images::Postgres;
use rustainers::runner::{RunOption, Runner};

pub fn init_postgresql(pg_db: &str, pg_user: &str, pg_password: &str) -> (Cli, Postgres) {
    let image = Postgres::default()
        .with_db(pg_db)
        .with_user(user)
        .with_password(password);

    let podman = Runner::podman();
    (podman, image)
}
