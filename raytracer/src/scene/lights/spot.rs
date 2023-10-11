use crate::math::Vec3;

use super::light::{Light, attenuation_fn};




pub struct SpotLight {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
}


pub struct SpotLightRange {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
    pub range: f32,
}


impl Light for SpotLight {
    fn get_emission(&self, at_point: Vec3) -> Vec3 {
        let distance = (self.position - at_point).length();
        return attenuation_fn(distance, self.color * self.intensity);
    }
    // (distance, normal)
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3) {
        let vector = self.position - origin;
        (vector.length(), (vector).normalized())
    }
}


impl Light for SpotLightRange {
    fn get_emission(&self, at_point: Vec3) -> Vec3 {
        let distance = (self.position - at_point).length();
        return attenuation_fn(distance, self.color * self.intensity);
    }
    // (distance, normal)
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3) {
        let vector = self.position - origin;
        (vector.length(), (vector).normalized())
    }
}
