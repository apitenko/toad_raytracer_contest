use std::{any::Any, num::NonZeroUsize, thread::JoinHandle, time::Duration, path::PathBuf};

use crate::{
    scene::{
        scene::{Scene, TotallySafeSceneWrapper},
        workload::Workload,
    },
    surface::TotallySafeSurfaceWrapper,
    util::queue::Queue,
    worker_thread::WorkerThreadHandle,
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
    exit_handle_sender: std::sync::mpsc::Sender<bool>,
}

impl RenderThreadHandle {
    pub fn run(
        surface_wrapper: TotallySafeSurfaceWrapper,
        scene: *const Scene,
        output_filename: PathBuf,
    ) -> anyhow::Result<Self> {
        let scene = TotallySafeSceneWrapper::new(scene);
        let (exit_handle_sender, exit_handle) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            return Self::routine(surface_wrapper.clone(), scene, exit_handle, output_filename);
        });
        let rt = Self {
            thread,
            exit_handle_sender,
        };
        Ok(rt)
    }

    pub fn stop(self) {
        self.exit_handle_sender.send(true);
        self.thread.join();
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
        surface: TotallySafeSurfaceWrapper,
        scene: TotallySafeSceneWrapper,
        exit_handle: std::sync::mpsc::Receiver<bool>,
        output_filename: PathBuf
    ) -> anyhow::Result<Duration> {
        let start_frame_time = std::time::Instant::now();

        {
            let available_threads = unsafe {
                std::thread::available_parallelism().unwrap_or(NonZeroUsize::new_unchecked(12))
            };

            let mut task_queue = Queue::new();

            let total_pixels = surface.width() * surface.height();
            let total_tasks = (available_threads.get() * 20).min(total_pixels as usize);
            let pixels_per_task: usize = (total_pixels as f32 / total_tasks as f32).floor() as usize;

            for index in 0..(total_tasks - 1) {
                let workload = Workload::new(
                    (index * pixels_per_task) as u32,
                    ((index + 1) * pixels_per_task) as u32,
                    (surface.width(), surface.height()),
                );
                task_queue.get().push(workload).unwrap();
            }
            {
                let index = total_tasks - 1;
                let workload = Workload::new(
                    (index * pixels_per_task) as u32,
                    total_pixels,
                    (surface.width(), surface.height()),
                );
                task_queue.get().push(workload).unwrap();
            }

            let mut worker_thread_handles = Vec::new();

            for _ in 0..available_threads.get() {
                worker_thread_handles.push(WorkerThreadHandle::run(
                    surface.clone(),
                    task_queue.clone(),
                    scene.clone(),
                ));
            }

            // wait for comlpetion
            if cfg!(debug_assertions) {
                loop {
                    std::thread::sleep(Duration::from_millis(20));
                    if task_queue.get().is_empty() {
                        break;
                    }
                    if let Ok(d) = exit_handle.try_recv() {
                        // nuke threads
                        for item in worker_thread_handles {
                            unsafe {
                                stop_thread::kill_thread_forcibly_exit_code(item.thread, 0);
                            }
                        }
                        return Ok(Duration::from_secs(123456789));
                    }
                }
            }

            for item in worker_thread_handles {
                item.thread.join().unwrap();
            }
        }

        let end_frame_time = std::time::Instant::now();
        let frame_time_diff = end_frame_time - start_frame_time;

        let memory = unsafe { std::slice::from_raw_parts(surface.get() as *mut u8, surface.size_pixels() * 4) };

        println!("Saving to {}", output_filename.display());
        image::save_buffer(
            output_filename,
            memory,
            surface.width() * surface.scale(),
            surface.height() * surface.scale(),
            image::ColorType::Rgba8,
        )?;

        return Ok(frame_time_diff);
    }
}

pub struct RenderThreadContext {}

impl RenderThreadContext {
    pub fn new() -> Self {
        Self {}
    }
}
