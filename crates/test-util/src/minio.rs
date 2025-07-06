use app_config::storage_configuration::StorageConfiguration;
use rustainers::images::Minio;

pub fn init_minio() -> Result<(Runner, Minio), RunnerError> {
    let image = Minio::default();
    let podman = Runner::podman()?;
    Ok((podman, image))
}
