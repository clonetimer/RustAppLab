//! C1 学员工程：修改三个 TODO；main/model 和测试是提供的基础设施。
pub mod model;
pub use model::{Error, Job, Limits, Receipt};
use std::path::Path;
use tokio::sync::watch;

pub fn validate_jobs(_jobs: &[Job], _limits: Limits) -> Result<(), Error> {
    // TODO(1): 验证任务数量、并发、超时、正文大小、回环 URL 和跨平台文件名。
    Err(Error::Invalid("C1 TODO: validation".into()))
}
pub async fn fetch_one(_job: Job, _limits: Limits, _cancel: watch::Receiver<bool>) -> Result<Vec<u8>, Error> {
    // TODO(2): Reqwest 真正异步收取正文；拒绝代理/重定向，处理上限、超时、取消。
    Err(Error::Invalid("C1 TODO: fetch".into()))
}
pub async fn download_batch(_jobs: Vec<Job>, _destination: &Path, _limits: Limits, _cancel: watch::Receiver<bool>) -> Result<Vec<Receipt>, Error> {
    // TODO(3): 有界队列、稳定结果排序、暂存发布；错误/取消时收拢任务并清理。
    Err(Error::Invalid("C1 TODO: queue".into()))
}
