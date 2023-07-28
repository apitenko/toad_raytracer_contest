use std::thread::JoinHandle;

use crate::{constants::{RENDER_WIDTH, RENDER_HEIGHT}, render_thread::TotallySafeBufferMemoryWrapper};

pub struct Workload {
    current_pixel: u32,
    start_pixel: u32,
    end_pixel: u32,
}

impl Iterator for Workload {
    type Item = (u32, u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        self.current_pixel += 1;
        if self.current_pixel >= self.end_pixel {
            // stop, get some help
            return None;
        }
        else {
            return Some((
                self.current_pixel % RENDER_WIDTH,
                self.current_pixel / RENDER_WIDTH,
                self.current_pixel - self.start_pixel,
            ));
        }
    }
}



pub struct WorkerThreadHandle {
    thread: JoinHandle<()>,
}

impl WorkerThreadHandle {
    pub fn run(buffer: TotallySafeBufferMemoryWrapper, workload: Workload, /* ...scene ptr... */) -> Self {

        let thread = std::thread::spawn(move || {
            for (x, y, index) in workload {
                // Render a pixel
                let red = x % 255;
                let green = y % 255;
                let blue = (x * y) % 255;

                unsafe {
                    *buffer.memory.add(index as usize) = blue | (green << 8) | (red << 16);
                }
            }
        });
        Self {
            thread,
        }
    }
}
