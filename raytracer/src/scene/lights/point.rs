use crate::math::Vec3;

use super::light::{attenuation_fn, Light};

pub struct PointLight {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
}

impl PointLight {
    pub fn new(position: Vec3, intensity: f32, color: Vec3) -> Self {
        Self {
            position,
            intensity,
            color,
        }
    }
}

impl Light for PointLight {
    fn get_emission(&self, at_point: Vec3) -> Vec3 {
        let distance = (self.position - at_point).length();
        let attenuation = attenuation_fn(distance);
        return self.color * attenuation * self.intensity;
    }
    // (distance, normal)
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3) {
        let vector = self.position - origin;
        (vector.length(), (vector).normalized())
    }
}

/////////////////////////////////
///
pub struct PointLightRadius {
    pub position: Vec3,
    pub radius: f32,
    pub intensity: f32,
    pub color: Vec3,
}

impl PointLightRadius {
    pub fn new(position: Vec3, radius: f32, intensity: f32, color: Vec3) -> Self {
        Self {
            position,
            radius,
            intensity,
            color,
        }
    }
}

impl Light for PointLightRadius {
    fn get_emission(&self, at_point: Vec3) -> Vec3 {
        let distance = (self.position - at_point).length();
        let distance_in_radius = ((self.radius - distance) / self.radius);
        let attenuation = attenuation_fn(distance_in_radius);
        return self.color * attenuation * self.intensity;
    }
    // (distance, normal)
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3) {
        let vector = self.position - origin;
        (vector.length(), (vector).normalized())
    }
}
