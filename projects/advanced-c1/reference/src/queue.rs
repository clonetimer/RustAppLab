use crate::{fetch_one, transport::cancelled, validate_jobs, Error, Job, Limits, Receipt};
use std::{fs, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};
use tokio::{sync::watch, task::JoinSet};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Staging(PathBuf);
impl Drop for Staging { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
fn absent(path: &Path) -> Result<(), Error> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(Error::Invalid("destination already exists".into())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// All-or-nothing publication for a trusted local parent and a single writer.
/// Network tasks only return bounded bytes; they never write to staging files.
pub async fn download_batch(jobs: Vec<Job>, destination: &Path, limits: Limits, mut cancel: watch::Receiver<bool>) -> Result<Vec<Receipt>, Error> {
    validate_jobs(&jobs, limits)?;
    if *cancel.borrow() { return Err(Error::Cancelled); }
    absent(destination)?;
    let parent = destination.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = destination.file_name().ok_or_else(|| Error::Invalid("destination needs a directory name".into()))?;
    let name = name.to_string_lossy();
    let stage = loop {
        let path = parent.join(format!(".{name}.c1-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        match fs::create_dir(&path) {
            Ok(()) => break Staging(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    };
    let mut tasks = JoinSet::new();
    let mut pending = jobs.into_iter().enumerate();
    let mut receipts = Vec::new();
    let outcome: Result<(), Error> = async {
        loop {
            if *cancel.borrow() { return Err(Error::Cancelled); }
            while tasks.len() < limits.concurrency {
                let Some((index, job)) = pending.next() else { break; };
                let rx = cancel.clone();
                tasks.spawn(async move {
                    let name = job.name.clone();
                    let bytes = fetch_one(job, limits, rx).await?;
                    Ok::<_, Error>((index, name, bytes))
                });
            }
            if tasks.is_empty() { break; }
            let result = tokio::select! {
                biased;
                _ = cancelled(&mut cancel) => return Err(Error::Cancelled),
                result = tasks.join_next() => result,
            };
            let (index, name, bytes) = result.ok_or_else(|| Error::Task("missing task".into()))?
                .map_err(|e| Error::Task(e.to_string()))??;
            fs::write(stage.0.join(&name), &bytes)?;
            receipts.push((index, Receipt {name, bytes: bytes.len()}));
        }
        if *cancel.borrow() { return Err(Error::Cancelled); }
        absent(destination)?;
        fs::rename(&stage.0, destination)?;
        Ok(())
    }.await;
    // Abort then wait; do not return while owned network tasks are still running.
    if outcome.is_err() { tasks.shutdown().await; }
    outcome?;
    receipts.sort_by_key(|(index, _)| *index);
    Ok(receipts.into_iter().map(|(_, receipt)| receipt).collect())
}
