use std::thread::JoinHandle;

use crate::{
    constants::{COLOR_CALL_PARAMETERS, MISS_COLOR, MISS_COLOR_VEC3, RENDER_HEIGHT, RENDER_WIDTH},
    math::{random::random_in_unit_sphere, reflect, Ray, RayBounce, Vec3},
    scene::{
        scene::{Scene, TotallySafeSceneWrapper},
        workload::Workload,
    },
    surface::TotallySafeSurfaceWrapper,
    tracing::{ray_cast, MAX_BOUNCES, MULTISAMPLE_OFFSETS, MULTISAMPLE_SIZE},
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

                            // if u < 0.35 || u > 0.65 || v < 0.2 || v > 0.5 {
                            //     continue;
                            // }
                            // if u < 0.9 || u > 0.94 || v < 0.9 || v > 0.94 {
                            //     continue;
                            // }
                            let starting_ray = scene.camera.ray(u, v);

                            // Hit skybox (so it doesn't affect the lighting)
                            if scene.geometry.single_cast(starting_ray, true).is_missed() {
                                // first ray missed, get skybox color
                                // let unit_direction = starting_ray.direction().normalized();
                                // let skybox_color = scene.skybox.sample_from_direction(unit_direction);
                                // pixel_color += skybox_color;
                                // ray_color = MISS_COLOR_VEC3;
                                continue;
                            }

                            let ray_color = ray_cast(
                                RayBounce::default_from_ray(starting_ray),
                                scene,
                            );

                            pixel_color += ray_color;
                        }

                        pixel_color = pixel_color / MULTISAMPLE_SIZE as f32;

                        //scale??
                        pixel_color = pixel_color * 0.1;
                        pixel_color = pixel_color.clamp(0.0, 1.0);

                        surface.write((x, y), pixel_color);
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
