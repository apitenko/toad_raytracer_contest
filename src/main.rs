use std::num::NonZeroU32;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_OUTPUT: bool = true;
const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 900;
const RENDER_WIDTH: u32 = 400;
const RENDER_HEIGHT: u32 = 225;

const SCALE_FACTOR: f32 = 4.0;
const UPDATE_INTERVAL: f32 = 1.0 / 10.0;

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
        for index in 0..(width * height) {
            let y = index / width;
            let x = index % width;
            let red = x % 255;
            let green = y % 255;
            let blue = (x * y) % 255;

            buffer[index as usize] = blue | (green << 8) | (red << 16);
        }

        buffer.as_mut_ptr()
    };

    event_loop.run_return(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {

                let buffer = surface.buffer_mut().unwrap();
                buffer.present().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
