use crate::{validate_jobs, Error, Job, Limits};
use reqwest::{redirect::Policy, Client};
use tokio::{sync::watch, time::timeout};

pub(crate) async fn cancelled(cancel: &mut watch::Receiver<bool>) {
    loop {
        if *cancel.borrow_and_update() {
            return;
        }
        if cancel.changed().await.is_err() {
            // A dropped false sender means no more cancellation, not an error or spin loop.
            std::future::pending::<()>().await;
        }
    }
}
fn transport(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::Timeout
    } else {
        Error::Transport(error.to_string())
    }
}

/// Receive a bounded body; deadline covers headers AND the complete body.
/// A separate idle deadline bounds each wait for the next body chunk.
pub async fn fetch_one(
    job: Job,
    limits: Limits,
    mut cancel: watch::Receiver<bool>,
) -> Result<Vec<u8>, Error> {
    validate_jobs(std::slice::from_ref(&job), limits)?;
    if *cancel.borrow() {
        return Err(Error::Cancelled);
    }
    let client = Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .connect_timeout(limits.connect_timeout)
        .build()
        .map_err(transport)?;
    let transfer = async {
        let mut response = client.get(&job.url).send().await.map_err(transport)?;
        if !response.status().is_success() {
            return Err(Error::Http(response.status().as_u16()));
        }
        if response
            .content_length()
            .is_some_and(|n| n > limits.max_bytes as u64)
        {
            return Err(Error::TooLarge);
        }
        let mut bytes = Vec::new();
        loop {
            let chunk = timeout(limits.idle_timeout, response.chunk())
                .await
                .map_err(|_| Error::IdleTimeout)?
                .map_err(transport)?;
            match chunk {
                None => return Ok(bytes),
                Some(chunk) => {
                    if chunk.len() > limits.max_bytes - bytes.len() {
                        return Err(Error::TooLarge);
                    }
                    bytes.extend_from_slice(&chunk);
                }
            }
        }
    };
    tokio::select! {
        biased;
        _ = cancelled(&mut cancel) => Err(Error::Cancelled),
        result = timeout(limits.total_timeout, transfer) => result.map_err(|_| Error::Timeout)?,
    }
}
