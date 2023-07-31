use std::{any::Any, num::NonZeroUsize, thread::JoinHandle, time::Duration};

use crate::{
    scene::{
        scene::{Scene, TotallySafeSceneWrapper},
        workload::Workload,
    },
    surface::TotallySafeSurfaceWrapper,
    worker_thread::WorkerThreadHandle, util::queue::Queue,
};

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
    pub fn run(
        surface_wrapper: TotallySafeSurfaceWrapper,
        size: (u32, u32),
        scene: *const Scene,
    ) -> anyhow::Result<Self> {
        let scene = TotallySafeSceneWrapper::new(scene);
        let thread = std::thread::spawn(move || {
            return Self::routine(surface_wrapper.clone(), size, scene);
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
        memory: TotallySafeSurfaceWrapper,
        size: (u32, u32),
        scene: TotallySafeSceneWrapper,
    ) -> anyhow::Result<Duration> {
        let start_frame_time = std::time::Instant::now();

        {
            let available_threads = unsafe {
                std::thread::available_parallelism().unwrap_or(NonZeroUsize::new_unchecked(12))
            };

            let mut task_queue = Queue::new();

            let total_pixels = size.0 * size.1;
            let total_tasks = available_threads.get() * 10;
            let pixels_per_task: usize = (total_pixels as f32 / total_tasks as f32).ceil() as usize;


            for index in 0..(total_tasks-1) {
                let workload = Workload::new(
                    (index * pixels_per_task) as u32,
                    ((index + 1) * pixels_per_task) as u32,
                );
                task_queue.get().push(workload).unwrap();
            }
            {
                let index = total_tasks-1;
                let workload = Workload::new((index * pixels_per_task) as u32, total_pixels);
                task_queue.get().push(workload).unwrap();
            }

            let mut worker_thread_handles = Vec::new();

            for _ in 0..available_threads.get() {
                worker_thread_handles.push(WorkerThreadHandle::run(
                    memory.clone(),
                    task_queue.clone(),
                    scene.clone(),
                ));
            }

            for item in worker_thread_handles {
                item.thread.join().unwrap();
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
