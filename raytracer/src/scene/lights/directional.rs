use crate::math::Vec3;

use super::light::Light;

pub struct DirectionalLight {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: Vec3,
}

impl DirectionalLight {
    pub fn new(direction: Vec3, intensity: f32, color: Vec3) -> Self {
        Self {
            color,
            direction: direction.normalized(),
            intensity,
        }
    }
}

impl Light for DirectionalLight {
    fn get_emission(&self, at_point: Vec3) -> Vec3 {
        return self.intensity * self.color;
    }
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3) {
        (f32::MAX, -self.direction)
    }
}
