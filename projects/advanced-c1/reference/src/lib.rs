//! A bounded loopback HTTP downloader for learning real async I/O.
//!
//! ```
//! use netfetch_c1::{validate_jobs, Job, Limits};
//! let jobs = [Job { name: "lesson.txt".into(), url: "http://127.0.0.1:8080/lesson".into() }];
//! validate_jobs(&jobs, Limits::default())?;
//! # Ok::<(), netfetch_c1::Error>(())
//! ```
pub mod model;
mod queue;
mod transport;
pub use model::{Error, Job, Limits, Receipt};
pub use queue::download_batch;
use std::{collections::HashSet, time::Duration};
pub use transport::fetch_one;

/// Validate the entire plan before opening a socket or creating any files.
pub fn validate_jobs(jobs: &[Job], limits: Limits) -> Result<(), Error> {
    if jobs.is_empty() || jobs.len() > 64 {
        return Err(Error::Invalid("expected 1..=64 jobs".into()));
    }
    if !(1..=8).contains(&limits.concurrency) || !(1..=8 * 1024 * 1024).contains(&limits.max_bytes)
    {
        return Err(Error::Invalid(
            "concurrency 1..=8; max_bytes 1..=8388608".into(),
        ));
    }
    for duration in [
        limits.connect_timeout,
        limits.total_timeout,
        limits.idle_timeout,
    ] {
        if duration.is_zero() || duration > Duration::from_secs(60) {
            return Err(Error::Invalid("timeouts must be in (0, 60s]".into()));
        }
    }
    let mut names = HashSet::new();
    for job in jobs {
        let name = &job.name;
        let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
        let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'));
        if name.is_empty()
            || name.len() > 64
            || name.starts_with('.')
            || name.ends_with('.')
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || reserved
            || !names.insert(name.to_ascii_lowercase())
        {
            return Err(Error::Invalid(format!(
                "unsafe or duplicate filename: {name}"
            )));
        }
        let url = reqwest::Url::parse(&job.url).map_err(|e| Error::Invalid(e.to_string()))?;
        // host_str for IPv6 includes brackets; parse only numeric addresses.
        let host = url
            .host_str()
            .unwrap_or("")
            .trim_start_matches('[')
            .trim_end_matches(']');
        let address = host.parse::<std::net::IpAddr>().ok();
        if url.scheme() != "http"
            || !address.is_some_and(|ip| ip.is_loopback())
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.port_or_known_default() == Some(0)
        {
            return Err(Error::Invalid(
                "only literal-loopback HTTP without credentials or fragment is supported".into(),
            ));
        }
    }
    Ok(())
}
