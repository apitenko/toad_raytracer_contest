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

                let mut color = Vec3::ZERO;

                for offset in MULTISAMPLE_OFFSETS {
                    // Render a pixel
                    let u = (x as f32 + offset.0) / RENDER_WIDTH as f32;
                    let v = (y as f32 + offset.1) / RENDER_HEIGHT as f32;

                    let ray = scene.camera.ray(u, v);

                    let mut cast_iterator = scene.geometry.traverse(ray);
                    let cast_result = cast_iterator.next();

                    if let Some(cast_result) = cast_result {
                        color = color + cast_result.color;
                    } else {
                        // skybox has already been hit on the previous step
                        // no object to hit, skip
                        color = color + MISS_COLOR_VEC3;
                    }
                }

                color = color / MULTISAMPLE_SIZE as f32;

                let red: u32 = (255.99 * color.x()) as u32;
                let green: u32 = (255.99 * color.y()) as u32;
                let blue: u32 = (255.99 * color.z()) as u32;

                let output = blue | (green << 8) | (red << 16);
                surface.write((x, y), output);
            }
        });
        Self { thread }
    }
}
