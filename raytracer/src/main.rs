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

use crate::math::{Vec3, Mat44};
use crate::primitives::mesh::{BoundingBox, Mesh};
use crate::primitives::quad::Quad;
use crate::primitives::shape::Shape;
use crate::primitives::sphere::Sphere;
use crate::primitives::triangle::Triangle;
use crate::scene::gltf_importer::read_into_scene;
use crate::scene::lights::directional::DirectionalLight;
use crate::scene::lights::point::PointLight;
use crate::scene::material::{Material, MaterialShared};
use crate::scene::scene::Scene;
use crate::scene::texture::{Texture, TextureShared};
use crate::surface::TotallySafeSurfaceWrapper;
use crate::util::fill_gradient::fill_gradient_black_to_white;
use crate::util::fresnel_constants::FresnelConstants;

fn main() -> anyhow::Result<()> {
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

    fill_gradient_black_to_white(surface_wrapper.clone());

    // ! TEXTURES //////////////////////////////////////////
    let mut textures_list = Vec::<Box<Texture>>::new();
    let mut capture_texture = |texture: Box<Texture>| {
        let mat_ptr = texture.as_ref() as *const Texture;
        let mat_shared = TextureShared::new(mat_ptr);
        textures_list.push(texture);
        mat_shared
    };

    // const TEXTURE_WHITE_1X1_BASE64: &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAAXNSR0IArs4c6QAAAA1JREFUGFdj+P///38ACfsD/QVDRcoAAAAASUVORK5CYII=";
    // let texture_default = capture_texture(Box::new(Texture::new_from_base64(
    //     TEXTURE_WHITE_1X1_BASE64,
    // )?));
    // const TEXTURE_1X1_MAGENTA_BASE64: &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAAEnQAABJ0Ad5mH3gAAAAMSURBVBhXY/jP8B8ABAAB/4jQ/cwAAAAASUVORK5CYII=";
    // let texture_1x1_magenta = capture_texture(Box::new(Texture::new_from_base64(
    //     TEXTURE_1X1_MAGENTA_BASE64,
    // )?));
    let texture_skybox = capture_texture(Box::new(Texture::new_from_file(&Path::new(
        "./res/skybox.png",
    ))?));
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

    let mut scene = Box::new(Scene::new(texture_skybox.clone()));

    read_into_scene(scene.as_mut(), "./res/scene2_embedded.gltf")?;
    // read_into_scene(scene.as_mut(), "./res/duck_embedded.gltf")?;

    if true
    {
        let aabb = BoundingBox {
            min: Quad::DEFAULT_GEOMETRY[0],
            max: Quad::DEFAULT_GEOMETRY[2],
        };
        let bounding_sphere = aabb.bounding_sphere();
        
        let color_texture = Texture::make_default_texture()?;
        let color_albedo = scene.material_storage.push_texture(color_texture);
        let mat = Material {
            color_factor: Vec3::ONE,
            color_albedo,
            ..Default::default()
        };
        
        let mat_shared = scene.material_storage.push_material(mat);
        let translation_matrix = Mat44::from_translation([0.0, -1.0, 0.0]);
        
        scene.add_mesh(Mesh {
            aabb,
            bounding_sphere,
            material: mat_shared,
            triangles: vec![
                Triangle {
                    vertices: [
                        translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[0]),
                        translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[1]),
                        translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[2]),
                    ],
                },
                Triangle {
                    vertices: [
                        translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[0]),
                        translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[2]),
                        translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[3]),
                    ],
                },
            ],
        });
    }

    // for shape in &shapes_list {
    //     scene.push_shape(shape.as_ref() as *const dyn Shape);
    // }

    // ! LIGHTS //////////////////////////////////////
    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([2.5, 0.2, -0.8]),
    //     5.0,
    //     5.0,
    //     Vec3::from_rgb(255, 60, 255),
    // )));
    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([0.0, 7.0, -1.0]),
    //     250.0,
    //     0.5,
    //     COLOR_WHITE,
    // )));

    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([10.0, 10.0, -1.0]),
    //     25.0,
    //     1.0,
    //     COLOR_WHITE,
    // )));

    // scene.lights.push(Box::new(PointLight::new(
    //     Vec3::new([0.0, 20.02, 0.0]),
    //     125.0,
    //     1.4,
    //     COLOR_WHITE,
    // )));

    scene.lights.push(Box::new(DirectionalLight::new(
        Vec3::new([0.5, -1.0, 0.0]),
        0.001,
        COLOR_SKY_BLUE,
    )));

    // notice the intentional lack of thread synchronization. for the moment.
    let unsafe_scene_ptr: *const Scene = scene.as_ref();

    let mut render_thread: Cell<Option<RenderThreadHandle>> = Cell::new(Some(
        RenderThreadHandle::run(surface_wrapper.clone(), RENDER_SIZE, unsafe_scene_ptr)
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

    Ok(())
}
