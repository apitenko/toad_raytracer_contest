use std::f32::consts::PI;

use crate::math::Vec3;

pub struct Skybox {}

impl Skybox {
    pub fn uv(direction: Vec3) -> (f32, f32) {
        let theta = f32::acos(direction.y()) / -PI;
        let phi = f32::atan2(direction.x(), -direction.z()) / -PI * 0.5;
        return (phi, theta);
    }
}
