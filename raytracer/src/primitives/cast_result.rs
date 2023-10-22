use std::mem::MaybeUninit;

use crate::{
    constants::COLOR_CALL_PARAMETERS,
    math::{Ray, Vec3},
    scene::material::MaterialShared,
};

use super::triangle::Triangle;

pub struct CastResult {
    pub distance_traversed: f32,
    pub intersection_point: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv: [(f32, f32); 4],
    pub material: MaterialShared,
    // pub triangle: Triangle
}

impl CastResult {
    pub const MISS: Self = Self {
        intersection_point: Vec3::ZERO,
        normal: Vec3::ZERO,
        tangent: Vec3::ZERO,
        bitangent: Vec3::ZERO,
        distance_traversed: f32::INFINITY,
        uv: [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
        material: MaterialShared::null(),
        // triangle: unsafe {MaybeUninit::zeroed().assume_init()}
    };

    #[inline]
    pub fn has_missed(&self) -> bool {
        return self.distance_traversed == f32::INFINITY;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ConeCastResult {
    pub accumulated_color: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct ConeCastResultStep {
    pub accumulated_color: Vec3,
    pub accumulated_density: f32, // see nvidia vxgi
}

impl ConeCastResultStep {
    pub fn empty() -> Self {
        Self {
            accumulated_color: Vec3::ZERO,
            accumulated_density: 0.0,
        }
    }

    pub fn merge(left: Self, right: Self) -> Self {
        Self {
            accumulated_color: left.accumulated_color + right.accumulated_color,
            accumulated_density: left.accumulated_density + right.accumulated_density
        }
    }
}

#[deprecated]
fn skybox_color(ray: &Ray) {
    // "skybox"
    let ray_normalized = ray.direction().normalized();
    let t = 0.5 * (ray_normalized.y() + 1.0);
    (1.0 - t) * Vec3::ONE + t * COLOR_CALL_PARAMETERS;
}
