//! 异步任务控制器

use async_std::sync::{Arc, Mutex};
use log::info;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug)]
pub struct Semaphore {
    permits: Arc<Mutex<AtomicUsize>>,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            permits: Arc::new(Mutex::new(AtomicUsize::new(permits))),
        }
    }

    pub async fn acquire(&self) {
        let permits = self.permits.lock().await;
        while permits.load(Ordering::Relaxed) == 0 {
            info!("task pool full, waiting ...");
            async_std::task::yield_now().await;
        }

        permits.fetch_sub(1, Ordering::Relaxed);
    }

    pub async fn release(&self) {
        self.permits.lock().await.fetch_add(1, Ordering::Relaxed);
    }
}
