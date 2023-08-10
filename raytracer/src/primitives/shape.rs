use crate::{math::{Ray, Vec3}, scene::material::Material};

use super::cast_result::CastResult;

pub trait Shape {
    fn intersect(&self, ray: Ray, inside: bool) -> Option<CastResult>;
}