use crate::{math::Vec3, primitives::cast_result::CastResult};

pub trait Light {
    fn get_emission(&self, at_point: Vec3) -> Vec3;
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3);
}

const ATTENUATION_PARAMETERS: (f32, f32, f32) = (0.0, 0.0, 1.0);

pub fn attenuation_fn(distance: f32) -> f32 {
    return 1.0 / (ATTENUATION_PARAMETERS.0
        + ATTENUATION_PARAMETERS.1 * distance
        + ATTENUATION_PARAMETERS.2 * (distance * distance));
}