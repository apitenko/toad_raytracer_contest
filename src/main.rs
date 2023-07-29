#![feature(try_blocks)]

use fps_counter::FpsCounter;
use std::cell::Cell;
use std::num::NonZeroU32;
use std::time::Duration;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod async_util;
pub mod constants;
pub mod fps_counter;
pub mod render_thread;
pub mod worker_thread;

use constants::*;
use render_thread::*;

fn main() {
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let (width, height) = (RENDER_WIDTH, RENDER_HEIGHT);

    let unsafe_buffer_ptr = {
        surface
            .resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            )
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        // for index in 0..(width * height) {
        //     let y = index / width;
        //     let x = index % width;
        //     let red = x % 255;
        //     let green = y % 255;
        //     let blue = (x * y) % 255;

        //     buffer[index as usize] = blue | (green << 8) | (red << 16);
        // }

        buffer.as_mut_ptr()
    };

    let mut render_thread: Cell<Option<RenderThreadHandle>> = Cell::new(Some(
        RenderThreadHandle::run(unsafe_buffer_ptr, RENDER_SIZE)
            .expect("RenderThreadHandle cannot start"),
    ));
    let mut fps_counter = FpsCounter::new();
    const TRY_INTERVAL_MAX: f32 = 1.0;
    let mut try_interval: f32 = 0.0;

    event_loop.run_return(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                //render_thread.stop();
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                let frame_time = fps_counter.update();
                try_interval -= frame_time.delta;
                if try_interval <= 0.0 {
                    try_interval += TRY_INTERVAL_MAX;
                    let rt = render_thread.replace(None);
                    if let Some(rt) = rt {
                        let result = rt.check_finished();
                        match result {
                            IsFinished::Continue(continued_render_thread) => {
                                render_thread.replace(Some(continued_render_thread));
                            },
                            IsFinished::Finished(data) => {
                                let duration = match data {
                                    Err(e) => return (),
                                    Ok(r) => match r {
                                        Err(e) => return (),
                                        Ok(duration) => duration,
                                    }
                                };

                                println!("Frame rendered in {:?}", duration);
                            }
                        }
                    }
                }

                let buffer = surface.buffer_mut().unwrap();
                buffer.present().unwrap();
            }
            _ => {}
        }
    });
}
