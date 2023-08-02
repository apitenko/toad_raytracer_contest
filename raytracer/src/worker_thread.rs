use std::thread::JoinHandle;

use crate::{
    constants::{COLOR_CALL_PARAMETERS, MISS_COLOR, MISS_COLOR_VEC3, RENDER_HEIGHT, RENDER_WIDTH},
    math::{random::random_in_unit_sphere, reflect, Ray, RayBounce, Vec3},
    scene::{
        scene::{Scene, TotallySafeSceneWrapper},
        workload::Workload,
    },
    surface::TotallySafeSurfaceWrapper,
    tracing::{outside_cast, MAX_BOUNCES, MULTISAMPLE_OFFSETS, MULTISAMPLE_SIZE},
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

                            // first hit to eliminate branching in loop
                            // if scene.geometry.single_cast(starting_ray).is_missed() {
                            //     // first ray missed, get skybox color
                            //     let unit_direction = starting_ray.direction().normalized();
                            //     let skybox_color = scene.skybox.sample_from_direction(unit_direction);
                            //     // let t = 0.5 * (unit_direction.y() + 1.0);
                            //     // pixel_color += (1.0 - t) * Vec3::ONE + t * COLOR_CALL_PARAMETERS;
                            //     pixel_color += skybox_color;
                            //     // ray_color = MISS_COLOR_VEC3;
                            //     continue;
                            // }

                            let indirect_lighting = outside_cast(
                                RayBounce {
                                    ray: starting_ray,
                                    bounces: MAX_BOUNCES,
                                    multiplier: 1.0,
                                },
                                scene,
                            );

                            pixel_color += indirect_lighting;
                        }

                        pixel_color = pixel_color / MULTISAMPLE_SIZE as f32;

                        //scale??
                        // pixel_color = pixel_color * 0.1;
                        // gamma correct
                        // pixel_color = pixel_color.clamp(0.0, 1.0);
                        pixel_color = pixel_color.gamma_correct_2();
                        // pixel_color = pixel_color.gamma_correct_2();
                        // pixel_color = pixel_color.gamma_correct_2();
                        // pixel_color = pixel_color.gamma_correct_2();

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
