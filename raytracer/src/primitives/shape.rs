use crate::{math::{Ray, Vec3}, scene::material::Material};

use super::cast_result::CastResult;

pub trait Shape {
    fn intersect(&self, ray: Ray, inside: bool) -> Option<CastResult>;
    fn material(&self) -> &Material;
    fn uv(&self, intersection_point: Vec3) -> (f32, f32);
}