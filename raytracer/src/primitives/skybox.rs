use std::f32::consts::PI;

use crate::{math::Vec3, constants::MISS_COLOR_VEC3, scene::texture::texture::TextureShared};

const SKYBOX_EMISSION_INTENSITY: f32 = 0.1;
pub struct Skybox {
    texture: TextureShared,
}

impl Skybox {
    fn uv(direction: Vec3) -> (f32, f32) {
        let theta = f32::acos(direction.y()) / -PI;
        let phi = f32::atan2(direction.x(), -direction.z()) / -PI * 0.5;
        return (phi, theta);
    }

    fn sample(&self, u: f32, v: f32) -> Vec3 {
        let sample = self.texture.get().sample(u, v);
        sample
    }

    pub fn sample_from_direction(&self, direction: Vec3) -> Vec3 {
        return MISS_COLOR_VEC3 / 3.0;

        // let t = 0.5 * (unit_direction.y() + 1.0);
        // pixel_color += (1.0 - t) * Vec3::ONE + t * COLOR_CALL_PARAMETERS;

        let (u,v) = Self::uv(direction);
        self.sample(u, v) * SKYBOX_EMISSION_INTENSITY
    }

    pub fn new(texture: TextureShared) -> Self {
        Self { texture }
    }
}
