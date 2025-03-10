use app_config::database_configuration::DatabaseConfiguration;
use rustainers::images::Postgres;
use rustainers::runner::Runner;
use rustainers::runner::RunnerError;
use rustainers::ExposedPort;

pub type PostgresContainer = rustainers::Container<Postgres>;

pub fn init_postgresql(config: &DatabaseConfiguration) -> Result<(Runner, Postgres), RunnerError> {
    let image = Postgres::default()
        .with_db(config.pg_db.as_str())
        .with_user(config.pg_user.as_str())
        .with_password(config.pg_password.as_str())
        .with_port(ExposedPort::fixed(config.pg_port, config.pg_port));

    let podman = Runner::podman()?;
    Ok((podman, image))
}
