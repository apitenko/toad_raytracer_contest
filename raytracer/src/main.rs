#![allow(non_snake_case)]
#![feature(try_blocks)]
#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]
#![feature(stdarch)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(const_try)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(inline_const)]
#![feature(negative_impls)]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]
#![feature(new_uninit)]
#![feature(thread_local)]

use fps_counter::FpsCounter;
use std::cell::Cell;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod async_util;
pub mod cli_api;
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

use crate::scene::gltf_importer::read_into_scene;
use crate::scene::scene::Scene;
use crate::scene::scene_defaults::add_scene_defaults;
use crate::surface::TotallySafeSurfaceWrapper;
use crate::util::fill_gradient::fill_gradient_black_to_white;

fn main() -> anyhow::Result<()> {
    println!("Raytracer init...");
    println!("Parsing CLI...");
    // CLI
    let cli = cli_api::cli_parse();

    let height = cli.height.parse::<u32>().unwrap_or_else(|e| {
        println!(
            "Can't parse height, using default height of {}. Encountered error: {}",
            DEFAULT_HEIGHT, e
        );
        DEFAULT_HEIGHT
    });
    let input = cli.input.to_str().unwrap();
    let output = cli.output;
    let stay_after_complete = cli.stay_after_complete;
    let camera_name = cli.camera_name.as_str();

    println!("Parsing scene from {input}...");
    // Scene
    let mut scene = Box::new(Scene::new()?);
    read_into_scene(scene.as_mut(), input, camera_name)?;
    add_scene_defaults(scene.as_mut())?;
    println!("Scene read! Creating window...");

    // Render target setup
    let aspect_ratio = scene.aspect_ratio();
    let width = (height as f32 * aspect_ratio).round() as u32;

    let render_size = (width, height);
    let render_scale = RENDER_SCALE;
    let window_size = (render_size.0 * render_scale, render_size.1 * render_scale);

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A duck is fine too")
        .with_inner_size(winit::dpi::PhysicalSize::new(window_size.0, window_size.1))        
        .build(&event_loop)
        .unwrap();

    println!("Creating window surface...");
    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let unsafe_buffer_ptr = {
        surface
            .resize(
                NonZeroU32::new(window_size.0).unwrap(),
                NonZeroU32::new(window_size.1).unwrap(),
            )
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        buffer.as_mut_ptr()
    };

    let surface_wrapper =
        TotallySafeSurfaceWrapper::new(unsafe_buffer_ptr, render_size, render_scale);

    println!("Filling surface with pain and sadness...");
    fill_gradient_black_to_white(surface_wrapper.clone());

    // ! TEXTURES //////////////////////////////////////////

    // const TEXTURE_WHITE_1X1_BASE64: &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAAXNSR0IArs4c6QAAAA1JREFUGFdj+P///38ACfsD/QVDRcoAAAAASUVORK5CYII=";
    // let texture_default = capture_texture(Box::new(Texture::new_from_base64(
    //     TEXTURE_WHITE_1X1_BASE64,
    // )?));
    // const TEXTURE_1X1_MAGENTA_BASE64: &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAAEnQAABJ0Ad5mH3gAAAAMSURBVBhXY/jP8B8ABAAB/4jQ/cwAAAAASUVORK5CYII=";
    // let texture_1x1_magenta = capture_texture(Box::new(Texture::new_from_base64(
    //     TEXTURE_1X1_MAGENTA_BASE64,
    // )?));
    // let texture_checkerboard = capture_texture(Box::new(Texture::new_from_file(&Path::new(
    //     "./res/checkerboard.png",
    // ))?));
    // let texture_concrete = capture_texture(Box::new(Texture::new_from_file(&Path::new(
    //     "./res/concrete.jpg",
    // ))?));

    // ! MATERIALS //////////////////////////////////////

    // let mut materials_list = Vec::<Box<Material>>::new();

    // let mut capture_material = |mat: Box<Material>| {
    //     let mat_ptr = mat.as_ref() as *const Material;
    //     let mat_shared = MaterialShared::new(mat_ptr);
    //     materials_list.push(mat);
    //     mat_shared
    // };

    // let fresnel_floor = 1.00089;
    // let fresnel_balls = 3.0;

    // let material_test_inside_intersections = capture_material(Box::new(Material {
    //     uv_scale: 1.0,
    //     color_factor: Vec3::from_rgb(255, 255, 255),
    //     color_albedo: texture_concrete.clone(),
    //     fresnel_coefficient: 9.9,
    //     roughness: 0.211,
    //     ..Default::default()
    // }));

    // let floor_checkerboard = capture_material(Box::new(Material {
    //     uv_scale: 0.01,
    //     color_factor: Vec3::ONE,
    //     color_albedo: texture_concrete.clone(),
    //     fresnel_coefficient: 4.0,
    //     roughness: 0.1,
    //     specular: 0.09 * Vec3::ONE,
    //     ..Default::default()
    // }));
    // let diffuse_green = capture_material(Box::new(Material {
    //     uv_scale: 1.0,
    //     color_factor: Vec3::from_rgb(10, 255, 10),
    //     color_albedo: texture_concrete.clone(),
    //     fresnel_coefficient: 1.03,
    //     roughness: 0.99,
    //     specular: 0.99 * Vec3::ONE,
    //     ..Default::default()
    // }));
    // let glass_blue = capture_material(Box::new(Material {
    //     uv_scale: 1.0,
    //     color_factor: COLOR_BLUE_SCUFF,
    //     color_albedo: texture_concrete.clone(),
    //     fresnel_coefficient: 3.0,
    //     roughness: 0.99,
    //     specular: 0.5 * Vec3::ONE,
    //     ..Default::default()
    // }));
    // let middle_red = capture_material(Box::new(Material {
    //     uv_scale: 1.0,
    //     color_factor: COLOR_RED_SCUFF,
    //     color_albedo: texture_concrete.clone(),
    //     fresnel_coefficient: 1.009,
    //     roughness: 0.779,
    //     specular: 0.02 * Vec3::ONE,
    //     ..Default::default()
    // }));

    // ! SHAPES //////////////////////////////////////
    // owning shapes container (Scene doesn't own shapes!)
    // let mut shapes_list = Vec::<Box<dyn Shape>>::new();

    // shapes_list.push(Box::new(Sphere::new(
    //     // "veryvery smol"
    //     Vec3::new([0.33, -0.45, -0.8]),
    //     0.02,
    //     smol.clone(),
    // )));
    // shapes_list.push(Box::new(Sphere::new(
    //     // "RED"
    //     Vec3::new([-0.2, 0.0, -1.0]),
    //     0.5,
    //     middle_red.clone(),
    // )));
    // shapes_list.push(Box::new(Sphere::new(
    //     // "GREEN"
    //     Vec3::new([0.6, -0.2, -1.0]),
    //     0.3,
    //     diffuse_green.clone(),
    // )));
    // shapes_list.push(Box::new(Sphere::new(
    //     // "BIG BLUE"
    //     Vec3::new([-3.0, 0.9, -3.0]),
    //     1.4,
    //     glass_blue.clone(),
    // )));

    // test inside intersection w/ Sphere

    // shapes_list.push(Box::new(Sphere::new(
    //     // "HOLLOW SPHERE"
    //     Vec3::ZERO,
    //     2.0,
    //     material_test_inside_intersections.clone(),
    // )));

    // shapes_list.push(Box::new(Sphere::new(
    //     // "FLOOR"
    //     Vec3::new([0.0, -100.5, -1.0]),
    //     100.0,
    //     default_material.clone(),
    // )));

    // QUAD
    // shapes_list.push(Box::new(Triangle::new(
    //     // "FLOOR"
    //     Vec3::new([0.0, -0.5, 0.0]),
    //     [
    //         Quad::DEFAULT_GEOMETRY[0],
    //         Quad::DEFAULT_GEOMETRY[1],
    //         Quad::DEFAULT_GEOMETRY[2],
    //     ],
    //     floor_checkerboard.clone(),
    // )));
    // shapes_list.push(Box::new(Triangle::new(
    //     // "FLOOR"
    //     Vec3::new([0.0, -0.5, 0.0]),
    //     [
    //         Quad::DEFAULT_GEOMETRY[0],
    //         Quad::DEFAULT_GEOMETRY[2],
    //         Quad::DEFAULT_GEOMETRY[3],
    //     ],
    //     floor_checkerboard.clone(),
    // )));

    // for shape in &shapes_list {
    //     scene.push_shape(shape.as_ref() as *const dyn Shape);
    // }

    // ! LIGHTS //////////////////////////////////////
    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([2.5, 0.2, -0.8]),
    //     500000.0,
    //     Vec3::from_rgb(255, 60, 255),
    // )));
    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([0.0, 7.0, -1.0]),
    //     250000.0,
    //     COLOR_BLUE,
    // )));

    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([10.0, 10.0, -1.0]),
    //     250000.0,
    //     COLOR_RED,
    // )));

    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([0.0, 20.02, 0.0]),
    //     125000.0,
    //     COLOR_GREEN,
    // )));

    // notice the intentional lack of thread synchronization. for the moment.
    let unsafe_scene_ptr: *const Scene = scene.as_ref();

    println!("Starting render threads...");
    #[allow(unused_mut)]
    let mut render_thread: Cell<Option<RenderThreadHandle>> = Cell::new(Some(
        RenderThreadHandle::run(surface_wrapper.clone(), unsafe_scene_ptr, output)
            .expect("RenderThreadHandle cannot start"),
    ));
    let mut fps_counter = FpsCounter::new();
    const TRY_INTERVAL_MAX: f32 = 1.0;
    let mut try_interval: f32 = 0.0;

    println!("Event loop reached...");
    event_loop.run_return(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                std::process::exit(0);

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
                                    Err(e) => {
                                        println!("{:?}", e);
                                        return;
                                    }
                                    Ok(r) => match r {
                                        Err(e) => {
                                            println!("{}", e);
                                            return;
                                        }
                                        Ok(duration) => duration,
                                    },
                                };

                                println!("Frame rendered in {:?}", duration);
                                if !stay_after_complete {
                                    std::process::exit(0);
                                }
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

    Ok(())
}
