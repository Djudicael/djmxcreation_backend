use rustainers::images::Postgres;
use rustainers::runner::Runner;
use rustainers::runner::RunnerError;
use rustainers::ExposedPort;

pub type PostgresContainer = rustainers::Container<Postgres>;

const TEST_DB_USER: &str = "postgres";
const TEST_DB_PASSWORD: &str = "postgres";
const TEST_DB_NAME: &str = "portfolio";
const TEST_DB_PORT: u16 = 5432;

pub fn init_postgresql() -> Result<(Runner, Postgres), RunnerError> {
    let image = Postgres::default()
        .with_db(TEST_DB_NAME)
        .with_user(TEST_DB_USER)
        .with_password(TEST_DB_PASSWORD)
        .with_port(ExposedPort::fixed(TEST_DB_PORT, TEST_DB_PORT));

    let podman = Runner::podman()?;
    Ok((podman, image))
}
