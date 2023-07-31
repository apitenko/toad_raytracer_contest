#![feature(try_blocks)]
#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]

use fps_counter::FpsCounter;
use std::cell::Cell;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod async_util;
pub mod constants;
pub mod fps_counter;
pub mod math;
pub mod primitives;
pub mod render_thread;
pub mod scene;
pub mod surface;
pub mod tracing;
pub mod util;
pub mod worker_thread;

use constants::*;
use render_thread::*;

use crate::math::Vec3;
use crate::primitives::shape::Shape;
use crate::primitives::sphere::Sphere;
use crate::scene::lights::directional::DirectionalLight;
use crate::scene::lights::point::PointLight;
use crate::scene::material::{Lambertian, Material, MaterialShared, Metal};
use crate::scene::scene::Scene;
use crate::surface::TotallySafeSurfaceWrapper;

fn main() {
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let unsafe_buffer_ptr = {
        surface
            .resize(
                NonZeroU32::new(WINDOW_WIDTH).unwrap(),
                NonZeroU32::new(WINDOW_HEIGHT).unwrap(),
            )
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        buffer.as_mut_ptr()
    };

    let surface_wrapper =
        TotallySafeSurfaceWrapper::new(unsafe_buffer_ptr, RENDER_SIZE, SCALE_FACTOR);

    let mut materials_list = Vec::<Box<dyn Material>>::new();

    let lambertian_white = {
        let mat = Box::new(Lambertian::new(Vec3::new([1.0, 1.0, 1.0])));
        let mat_ptr = mat.as_ref() as *const dyn Material;
        let mat_shared = MaterialShared::new(mat_ptr);
        materials_list.push(mat);
        mat_shared
    };

    let lambertian_red = {
        let mat = Box::new(Lambertian::new(Vec3::new([1.0, 0.0, 0.0])));
        let mat_ptr = mat.as_ref() as *const dyn Material;
        let mat_shared = MaterialShared::new(mat_ptr);
        materials_list.push(mat);
        mat_shared
    };

    let lambertian_green = {
        let mat = Box::new(Lambertian::new(Vec3::new([0.0, 1.0, 0.0])));
        let mat_ptr = mat.as_ref() as *const dyn Material;
        let mat_shared = MaterialShared::new(mat_ptr);
        materials_list.push(mat);
        mat_shared
    };

    let lambertian_blue = {
        let mat = Box::new(Lambertian::new(Vec3::new([0.0, 0.0, 1.0])));
        let mat_ptr = mat.as_ref() as *const dyn Material;
        let mat_shared = MaterialShared::new(mat_ptr);
        materials_list.push(mat);
        mat_shared
    };

    let metal_white = {
        let mat = Box::new(Metal::new(Vec3::new([1.0, 1.0, 1.0])));
        let mat_ptr = mat.as_ref() as *const dyn Material;
        let mat_shared = MaterialShared::new(mat_ptr);
        materials_list.push(mat);
        mat_shared
    };

    // owning shapes container (Scene doesn't own shapes!)
    let mut shapes_list = Vec::<Box<Sphere>>::new();
    shapes_list.push(Box::new(Sphere::new(
        Vec3::new([-0.2, 0.0, -1.0]),
        0.5,
        lambertian_red.clone(),
    )));
    shapes_list.push(Box::new(Sphere::new(
        Vec3::new([0.0, -100.5, -1.0]),
        100.0,
        lambertian_green.clone(),
    )));
    shapes_list.push(Box::new(Sphere::new(
        Vec3::new([0.6, -0.2, -1.0]),
        0.3,
        metal_white.clone(),
    )));
    shapes_list.push(Box::new(Sphere::new(
        Vec3::new([-3.0, 0.9, -3.0]),
        1.4,
        lambertian_blue.clone(),
    )));

    let mut scene = Box::new(Scene::new());
    for shape in &shapes_list {
        scene.push_shape(shape.as_ref() as *const dyn Shape);
    }

    scene.lights.push(Box::new(PointLight::new(
        Vec3::new([0.0, 1.1, -1.0]),
        0.3,
        1.0,
        Vec3::new([0.5, 0.5, 1.0]),
    )));

    scene.lights.push(Box::new(PointLight::new(
        Vec3::new([0.0, 10.0, -1.0]),
        25.0,
        0.5,
        COLOR_WHITE,
    )));

    scene.lights.push(Box::new(PointLight::new(
        Vec3::new([10.0, 10.0, -1.0]),
        25.0,
        0.1,
        COLOR_WHITE,
    )));

    scene.lights.push(Box::new(DirectionalLight::new(
        Vec3::new([1.0, -1.0, 0.0]),
        0.1,
        COLOR_WHITE,
    )));

    // notice the intentional lack of thread synchronization. for the moment.
    let unsafe_scene_ptr: *const Scene = scene.as_ref();

    let mut render_thread: Cell<Option<RenderThreadHandle>> = Cell::new(Some(
        RenderThreadHandle::run(surface_wrapper, RENDER_SIZE, unsafe_scene_ptr)
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
                let rt = render_thread.replace(None);
                if let Some(rt) = rt {
                    rt.stop();
                }
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
                            }
                            IsFinished::Finished(data) => {
                                let duration = match data {
                                    Err(e) => return (),
                                    Ok(r) => match r {
                                        Err(e) => return (),
                                        Ok(duration) => duration,
                                    },
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
