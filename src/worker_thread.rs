use std::thread::JoinHandle;

use crate::{
    constants::{MISS_COLOR, RENDER_HEIGHT, RENDER_WIDTH},
    math::{Ray, Vec3},
    scene::{scene::TotallySafeSceneWrapper, workload::Workload},
    surface::TotallySafeSurfaceWrapper,
    tracing::color,
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
                // Render a pixel
                let u = x as f32 / RENDER_WIDTH as f32;
                let v = y as f32 / RENDER_HEIGHT as f32;

                let scene = unsafe { &(*scene.get()) };

                let ray = scene.camera.ray(u, v);

                let mut cast_iterator = scene.geometry.traverse(ray);
                let cast_result = cast_iterator.next();

                if let Some(cast_result) = cast_result {
                    let col = cast_result.color;
                    let red: u32 = (255.99 * col.x()) as u32;
                    let green: u32 = (255.99 * col.y()) as u32;
                    let blue: u32 = (255.99 * col.z()) as u32;
                    // let red = x % 255;
                    // let green = y % 255;
                    // let blue = (x * y) % 255;

                    let output = blue | (green << 8) | (red << 16);
                    surface.write((x, y), output);
                } else {
                    // skybox has already been hit on the previous step
                    // no object to hit
                }
            }
        });
        Self { thread }
    }
}
