use std::thread::JoinHandle;

use crate::{
    constants::{
        COLOR_CALL_PARAMETERS, MAX_BOUNCES, MISS_COLOR, MISS_COLOR_VEC3, RENDER_HEIGHT,
        RENDER_WIDTH,
    },
    math::{Ray, Vec3},
    scene::{scene::TotallySafeSceneWrapper, workload::Workload},
    surface::TotallySafeSurfaceWrapper,
    tracing::{MULTISAMPLE_OFFSETS, MULTISAMPLE_SIZE},
    util::queue::Queue,
};

pub struct WorkerThreadHandle {
    pub thread: JoinHandle<()>,
}

impl WorkerThreadHandle {
    pub fn run(
        mut surface: TotallySafeSurfaceWrapper,
        mut queue: Queue<Workload>,
        scene: TotallySafeSceneWrapper,
    ) -> Self {
        let thread = std::thread::spawn(move || {
            loop {
                let new_task = queue.get().pop();
                if let Ok(workload) = new_task {
                    for (x, y, _) in workload {
                        let scene = unsafe { &(*scene.get()) };

                        let mut pixel_color = Vec3::ZERO;

                        for offset in MULTISAMPLE_OFFSETS {
                            // Render a pixel
                            let u = (x as f32 + offset.0) / RENDER_WIDTH as f32;
                            let v = (y as f32 + offset.1) / RENDER_HEIGHT as f32;

                            let starting_ray = scene.camera.ray(u, v);
                            let mut current_ray = starting_ray;

                            let mut ray_color = Vec3::ZERO;
                            let mut bounces = 0;

                            while bounces < MAX_BOUNCES {
                                bounces += 1;

                                let cast_result = scene.geometry.single_cast(current_ray);
                                let (reflection, attenuation, is_hit) = cast_result
                                    .material
                                    .get()
                                    .scatter(&current_ray, &cast_result);

                                if cast_result.distance_traversed == f32::INFINITY {
                                    break;
                                }
                                if !is_hit {
                                    break;
                                }

                                current_ray = reflection;

                                // scene lighting
                                for light_source in &scene.lights {
                                    let (distance_to_light, normal_into_light) =
                                        light_source.normal_from(cast_result.intersection_point);

                                    let light_cast_result = scene.geometry.single_cast(Ray::new(
                                        cast_result.intersection_point,
                                        normal_into_light,
                                        distance_to_light,
                                    ));
                                    if light_cast_result.is_missed() {
                                        let light_color = light_source
                                            .get_emission(cast_result.intersection_point);
                                        ray_color = ray_color + light_color * attenuation;
                                    }

                                    // TODO: apply material
                                }
                                // ray_color = ray_color + cast_result.color;
                            }

                            if bounces == 1 {
                                let unit_direction = starting_ray.direction().normalized();
                                let t = 0.5 * (unit_direction.y() + 1.0);
                                ray_color = (1.0 - t) * Vec3::ONE + t * COLOR_CALL_PARAMETERS;
                                // ray_color = MISS_COLOR_VEC3;
                            } else {
                                ray_color = ray_color / bounces as f32;
                            }

                            pixel_color = pixel_color + ray_color;
                        }

                        pixel_color = pixel_color / MULTISAMPLE_SIZE as f32;

                        // gamma correct
                        pixel_color = pixel_color.gamma_correct_2();
                        // pixel_color = pixel_color.clamp(0.0, 1.0);

                        let red: u32 = (255.99 * pixel_color.x()) as u32;
                        let green: u32 = (255.99 * pixel_color.y()) as u32;
                        let blue: u32 = (255.99 * pixel_color.z()) as u32;

                        let output = blue | (green << 8) | (red << 16);
                        surface.write((x, y), output);
                    }
                } else {
                    // no more work; exit
                    return;
                }
            }
        });
        Self { thread }
    }
}
