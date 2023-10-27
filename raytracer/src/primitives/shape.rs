use crate::{math::{Ray, Vec3}, scene::material::Material};

use super::cast_result::CastIntersectionResult;

pub trait Shape {
    fn intersect(&self, ray: Ray, inside: bool) -> Option<CastIntersectionResult>;
}