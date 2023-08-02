use std::f32::consts::PI;

use crate::{math::Vec3, scene::texture::TextureShared};

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
        let (u,v) = Self::uv(direction);
        self.sample(u, v) * SKYBOX_EMISSION_INTENSITY
    }

    pub fn new(texture: TextureShared) -> Self {
        Self { texture }
    }
}
