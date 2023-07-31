use crate::{math::Vec3, primitives::cast_result::CastResult};

const ATTENUATION_PARAMETERS: (f32, f32, f32) = (0.0, 0.1, 0.1);

fn attenuation_fn(input: f32) -> f32 {
    return (ATTENUATION_PARAMETERS.0
        + ATTENUATION_PARAMETERS.1 / input
        + ATTENUATION_PARAMETERS.2 / (input * input)).clamp(0.0, 1.0);
}

pub struct PointLight {
    pub position: Vec3,
    pub radius: f32,
    pub strength: f32,
    pub color: Vec3,
}

impl PointLight {
    pub fn new(position: Vec3, radius: f32, strength: f32, color: Vec3) -> Self {
        Self {
            position,
            radius,
            strength,
            color,
        }
    }

    // returns color
    pub fn get_emission(&self, cast_result: &CastResult) -> Vec3 {
        let distance = (self.position - cast_result.intersection_point).length();
        let distance_in_radius = ((self.radius - distance) / self.radius).clamp(0.0, 1.0);
        let attenuation = attenuation_fn(distance_in_radius);
        return self.color * attenuation * self.strength;
    }
}
