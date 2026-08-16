use rustainers::images::Minio;
use rustainers::runner::Runner;
use rustainers::runner::RunnerError;

pub type MinioContainer = rustainers::Container<Minio>;

pub fn init_minio() -> Result<(Runner, Minio), Box<RunnerError>> {
    let image = Minio::default();
    let podman = Runner::podman().map_err(Box::new)?;
    Ok((podman, image))
}
