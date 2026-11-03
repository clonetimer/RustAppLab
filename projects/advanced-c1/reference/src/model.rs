use std::{fmt, time::Duration};

#[derive(Clone, Debug)]
pub struct Job {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub concurrency: usize,
    pub connect_timeout: Duration,
    pub total_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            concurrency: 2,
            connect_timeout: Duration::from_secs(2),
            total_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(2),
            max_bytes: 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub name: String,
    pub bytes: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
    Http(u16),
    Transport(String),
    Timeout,
    IdleTimeout,
    TooLarge,
    Cancelled,
    Io(String),
    Task(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
