use crate::{math::{Vec3, Ray}, constants::{MISS_COLOR_VEC3, COLOR_CALL_PARAMETERS}};

pub struct CastResult {
    pub distance_traversed: f32,
    pub intersection_point: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

impl CastResult {
    pub const MISS: Self = Self {
        color: Vec3::ZERO,
        intersection_point: Vec3::ZERO,
        normal: Vec3::ZERO,
        distance_traversed: f32::INFINITY,
    };

    pub fn is_missed(&self) -> bool {
        return self.distance_traversed == f32::INFINITY;
    }
}

#[deprecated]
fn skybox_color(ray: &Ray) {    
    // "skybox"
    let ray_normalized = ray.direction().normalized();
    let t = 0.5 * (ray_normalized.y() + 1.0);
    (1.0 - t) * Vec3::ONE + t * COLOR_CALL_PARAMETERS;
}