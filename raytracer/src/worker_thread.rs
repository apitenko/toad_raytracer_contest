use std::thread::JoinHandle;

use crate::{
    constants::{MISS_COLOR, MISS_COLOR_VEC3, RENDER_HEIGHT, RENDER_WIDTH},
    math::{Ray, Vec3},
    scene::{scene::TotallySafeSceneWrapper, workload::Workload},
    surface::TotallySafeSurfaceWrapper,
    tracing::{MULTISAMPLE_OFFSETS, MULTISAMPLE_SIZE},
};

pub struct WorkerThreadHandle {
    thread: JoinHandle<()>,
}

impl WorkerThreadHandle {
    pub fn run(
        mut surface: TotallySafeSurfaceWrapper,
        workload: Workload,
        scene: TotallySafeSceneWrapper,
    ) -> Self {
        let thread = std::thread::spawn(move || {
            for (x, y, _) in workload {
                let scene = unsafe { &(*scene.get()) };

                let mut pixel_color = Vec3::ZERO;

                for offset in MULTISAMPLE_OFFSETS {
                    // Render a pixel
                    let u = (x as f32 + offset.0) / RENDER_WIDTH as f32;
                    let v = (y as f32 + offset.1) / RENDER_HEIGHT as f32;

                    let ray = scene.camera.ray(u, v);

                    let mut cast_iterator = scene.geometry.traverse(ray);
                    let mut ray_color = Vec3::ZERO;
                    let mut bounces = 0;
                    for cast_result in cast_iterator {
                        bounces += 1;
                        ray_color = ray_color + cast_result.color;
                    }

                    if bounces == 1 {
                        ray_color = MISS_COLOR_VEC3;
                    } else {
                        ray_color = ray_color / bounces as f32;
                    }

                    pixel_color = pixel_color + ray_color;
                }

                pixel_color = pixel_color / MULTISAMPLE_SIZE as f32;

                let red: u32 = (255.99 * pixel_color.x()) as u32;
                let green: u32 = (255.99 * pixel_color.y()) as u32;
                let blue: u32 = (255.99 * pixel_color.z()) as u32;

                let output = blue | (green << 8) | (red << 16);
                surface.write((x, y), output);
            }
        });
        Self { thread }
    }
}
