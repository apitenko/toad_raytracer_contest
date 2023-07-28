use std::{any::Any, num::NonZeroUsize, thread::JoinHandle, time::Duration};

use crate::{async_util::poll, worker_thread::WorkerThreadHandle};

#[derive(Clone, Copy)]
pub struct TotallySafeBufferMemoryWrapper {
    pub memory: *mut u32,
}

unsafe impl Send for TotallySafeBufferMemoryWrapper {}
unsafe impl Sync for TotallySafeBufferMemoryWrapper {}

/// Renders 1 frame into the given memory then exits.
pub fn run_render_thread() {

    // receive the buffer memory to write to
    // receive the scene
    // spawn worker threads, pass them workloads
    // wait for the full completion
    // exit
}

type JoinHandleType = JoinHandle<Result<Duration, anyhow::Error>>;

pub enum IsFinished<T> {
    Finished(T),
    Continue(RenderThreadHandle),
}

pub struct RenderThreadHandle {
    thread: JoinHandleType,
}

impl RenderThreadHandle {
    pub fn run(memory: *mut u32, size: (u32, u32)) -> anyhow::Result<Self> {
        let memory = TotallySafeBufferMemoryWrapper { memory };
        let thread = std::thread::spawn(move || {
            return Self::routine(memory, size);
        });
        let rt = Self { thread };
        Ok(rt)
    }

    pub fn stop(&mut self) {
        // THERE IS NO STOPPING US
        /*
                      .a'---'a.
              \-_     / -. .- \
               /_"-__| (@)^(@) |__---__-'
              __--___""-_\ /_-""____/_
             /       "":YHiHY;""     \
           .'  .        '''''         '.
         .'   '-'[]      | |    []'-)   '.

        */
    }

    pub fn check_finished(
        self,
    ) -> IsFinished<Result<anyhow::Result<Duration>, Box<dyn Any + Send>>> {
        if self.thread.is_finished() {
            let result = self.thread.join();
            return IsFinished::Finished(result);
        }

        return IsFinished::Continue(self);
    }

    pub fn routine(
        memory: TotallySafeBufferMemoryWrapper,
        size: (u32, u32),
    ) -> anyhow::Result<Duration> {
        let start_frame_time = std::time::Instant::now();

        {
            // let context = RenderThreadContext::new();

            let available_threads = unsafe {
                std::thread::available_parallelism().unwrap_or(NonZeroUsize::new_unchecked(12))
            };

            
            let worker_thread_handles = Vec::new();
            
            for index in 0..available_threads.get() {
                worker_thread_handles.push(WorkerThreadHandle::run());
            }


        }

        let end_frame_time = std::time::Instant::now();
        let frame_time_diff = end_frame_time - start_frame_time;
        return Ok(frame_time_diff);
    }
}

pub struct RenderThreadContext {}

impl RenderThreadContext {
    pub fn new() -> Self {
        Self {}
    }
}
