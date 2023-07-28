use std::{sync::Arc, task::Poll, thread::Thread};

use futures::{Future, FutureExt};
use std::thread;

/// A waker that wakes up the current thread when called.
pub struct ThreadWaker(std::thread::Thread);

impl ThreadWaker {
    pub fn from_thread(t: Thread) -> Self {
        return Self(t);
    }
}
impl std::task::Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

pub fn poll<R, T: Future<Output = R>>(future: &mut T) -> Poll<R>
where
    T: std::marker::Unpin,
{
    let mut accept_future = Box::pin(future);

    let t = thread::current();
    let waker = Arc::new(ThreadWaker(t)).into();
    let mut cx = std::task::Context::from_waker(&waker);

    let result = accept_future.poll_unpin(&mut cx);
    return result;
}
