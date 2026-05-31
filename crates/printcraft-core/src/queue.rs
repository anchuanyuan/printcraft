//! 打印任务队列
//!
//! 使用 tokio mpsc 实现异步任务队列。
//! 支持提交任务、取消任务、等待完成。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex, oneshot};

use crate::print_job::PrintJob;

/// 队列中的任务及其完成通知
struct QueuedJob {
    job: PrintJob,
    cancel_tx: Option<oneshot::Sender<()>>,
}

/// 打印任务队列
///
/// 线程安全的异步打印队列。
/// 调用方通过 `submit()` 提交任务，服务端通过 `recv()` 接收并处理。
#[derive(Clone)]
pub struct PrintQueue {
    /// 任务发送端
    tx: mpsc::Sender<PrintJob>,
    /// 任务接收端（Arc<Mutex> 允许 clone 后共享）
    rx: Arc<Mutex<mpsc::Receiver<PrintJob>>>,
    /// 活跃任务追踪（用于取消）
    active_jobs: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
}

impl PrintQueue {
    /// 创建新的打印队列
    ///
    /// # Arguments
    /// * `buffer_size` - 队列缓冲区大小
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self {
            tx,
            rx: Arc::new(Mutex::new(rx)),
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 提交打印任务到队列
    ///
    /// 返回任务 ID，可用于后续取消。
    pub async fn submit(&self, job: PrintJob) -> String {
        let job_id = job.id.clone();
        tracing::info!("提交打印任务: {} ({})", job.name, job_id);

        if let Err(e) = self.tx.send(job).await {
            tracing::error!("提交任务失败: {}", e);
        }

        job_id
    }

    /// 从队列接收下一个任务（服务端调用）
    ///
    /// 阻塞直到有新任务入队。
    pub async fn recv(&self) -> Option<PrintJob> {
        self.rx.lock().await.recv().await
    }

    /// 取消指定任务
    ///
    /// 如果任务正在处理中，会发送取消信号。
    pub async fn cancel(&self, job_id: &str) -> bool {
        let mut jobs = self.active_jobs.lock().await;
        if let Some(tx) = jobs.remove(job_id) {
            let _ = tx.send(());
            tracing::info!("已取消任务: {}", job_id);
            true
        } else {
            tracing::warn!("任务不存在或已完成: {}", job_id);
            false
        }
    }

    /// 获取当前队列中活跃任务数量
    pub async fn active_count(&self) -> usize {
        self.active_jobs.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_and_recv() {
        let queue = PrintQueue::new(10);
        let job = PrintJob::new("test_print");

        let job_id = queue.submit(job).await;
        assert!(!job_id.is_empty());

        let received = queue.recv().await;
        assert!(received.is_some());
        let received = received.unwrap();
        assert_eq!(received.name, "test_print");
        assert_eq!(received.id, job_id);
    }

    #[tokio::test]
    async fn test_queue_clone() {
        let queue = PrintQueue::new(10);
        let cloned = queue.clone();

        let job = PrintJob::new("cloned_queue_job");
        queue.submit(job).await;

        let received = cloned.recv().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().name, "cloned_queue_job");
    }
}
