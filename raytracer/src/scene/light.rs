use crate::{math::Vec3, primitives::cast_result::CastResult};

// Linear falloff
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
        let strength = ((self.radius - distance) / self.radius).clamp(0.0, 1.0) * self.strength;
        return self.color * strength;
    }
}
