use std::sync::Arc;

use concurrent_queue::ConcurrentQueue;

#[derive(Clone)]
pub struct Queue<T> {
    queue: Arc<ConcurrentQueue<T>>,
}

unsafe impl<T> Send for Queue<T> {}
unsafe impl<T> Sync for Queue<T> {}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(ConcurrentQueue::unbounded()),
        }
    }

    pub fn get(&mut self) -> &ConcurrentQueue<T> {
        &self.queue
    }
}

// impl<T> Clone for Queue<T> {
//     fn clone(&self) -> Self {
//         return Self {
//             queue: self.queue.clone(),
//         }
//     }
// }
