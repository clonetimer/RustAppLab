pub mod model;
pub use model::{Error, Job, Limits, Receipt};
use std::path::Path;
use tokio::sync::watch;

pub fn validate_jobs(_jobs: &[Job], _limits: Limits) -> Result<(), Error> {
    Err(Error::Invalid("C1 TODO: validation".into()))
}
pub async fn fetch_one(_job: Job, _limits: Limits, _cancel: watch::Receiver<bool>) -> Result<Vec<u8>, Error> {
    Err(Error::Invalid("C1 TODO: fetch".into()))
}
pub async fn download_batch(_jobs: Vec<Job>, _destination: &Path, _limits: Limits, _cancel: watch::Receiver<bool>) -> Result<Vec<Receipt>, Error> {
    Err(Error::Invalid("C1 TODO: queue".into()))
}
