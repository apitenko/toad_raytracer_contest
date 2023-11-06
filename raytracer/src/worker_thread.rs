use std::thread::JoinHandle;

use crate::constants::{MULTISAMPLE_OFFSETS, MULTISAMPLE_SIZE};
use crate::scene::acceleration_structure::acceleration_structure::AccelerationStructure;
use crate::{
    math::{RayBounce, Vec3},
    scene::{
        scene::{Scene, TotallySafeSceneWrapper},
        workload::Workload,
    },
    surface::TotallySafeSurfaceWrapper,
    tracing::ray_cast,
    util::queue::Queue,
};

// TODO: revisit tone mapping techniques
fn hable(x: f32) -> f32 {
    let A: f32 = 0.15;
    let B: f32 = 0.50;
    let C: f32 = 0.10;
    let D: f32 = 0.20;
    let E: f32 = 0.02;
    let F: f32 = 0.30;

    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

fn reinhard(x: f32) -> f32 {
    x / (x + 1.0)
}

fn f32_lerp(left: f32, right: f32, t: f32) -> f32 {
    left + (right - left) * t
}

fn tone_mapping_v2(color: Vec3) -> Vec3 {
    let length = color.length();
    let length = f32::clamp(length, 0.01, f32::INFINITY);
    let newlength = reinhard(length);
    let color = color / length * newlength;
    color
}

fn tone_mapping(color: Vec3) -> Vec3 {
    // https://computergraphics.stackexchange.com/questions/6307/tone-mapping-bright-images
    // Calculate the desaturation coefficient based on the brightness
    let sig = f32::max(color.x(), f32::max(color.y(), color.z()));
    let luma = color.luminosity();// Vec3::dot(color, Vec3::from_f32([0.2126, 0.7152, 0.0722, 0.0]));
    let coeff = f32::max(sig - 0.18, 1e-6) / f32::max(sig, 1e-6);
    let coeff = f32::powi(coeff, 20);

    // Update the original color and signal
    let color = Vec3::lerp(color, Vec3::from_f32([luma, luma, luma, 0.0]), coeff);
    let sig = f32_lerp(sig, luma, coeff);

    // Perform tone-mapping
    let mapping = hable(sig) / sig;
    let color = color * mapping;
    let color = color.clamp(0.0, 1.0);
    return color;
}

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
                            let u = (x as f32 + offset.0) / surface.width() as f32;
                            let v = (y as f32 + offset.1) / surface.height() as f32;

                            // if u < 0.81 || u > 0.91 || v < 0.39 || v > 0.69 {
                            //     continue;
                            // }
                            // Full metallic sphere
                            // if u < 0.31 || u > 0.36 || v < 0.75 || v > 0.85 {
                            //     continue;
                            // }

                            // if u < 0.36 || u > 0.41 || v < 0.65 || v > 0.75 {
                            //     continue;
                            // }

                            // if u < 0.3 || u > 0.7 {
                            //     continue;
                            // }
                            // if u < 0.3 || u > 0.7 || v < 0.55 || v > 0.65 {
                            //     continue;
                            // }
                            // if u < 0.9 || u > 0.94 || v < 0.9 || v > 0.94 {
                            //     continue;
                            // }
                            // if u < 0.45 || u > 0.47 || v < 0.19 || v > 0.2 {
                            //     continue;
                            // }
                            let starting_ray = scene.camera.ray(u, v);

                            // TODO: skybox
                            // Hit skybox (so it doesn't affect the lighting)
                            if scene.geometry.single_cast(starting_ray, true).has_missed() {
                                // first ray missed, get skybox color
                                // let unit_direction = starting_ray.direction().normalized();
                                // let skybox_color = scene.skybox.sample_from_direction(unit_direction);
                                // pixel_color += skybox_color;
                                // ray_color = MISS_COLOR_VEC3;
                                // pixel_color += SKYBOX_COLOR * 10000.0;
                                continue;
                            }

                            let ray_color =
                                ray_cast(RayBounce::default_from_ray(starting_ray), scene);

                            pixel_color += ray_color;
                        }

                        pixel_color = pixel_color / MULTISAMPLE_SIZE as f32;

                        // pixel_color = pixel_color * 5.0;
                        // pixel_color = pixel_color / 400.0;

                        // let lumi = pixel_color.luminosity();
                        // if lumi > 10.0 {
                        //     pixel_color = pixel_color / lumi;
                        // }

                        pixel_color = tone_mapping(pixel_color);
                        pixel_color = pixel_color.gamma_correct_2();

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
