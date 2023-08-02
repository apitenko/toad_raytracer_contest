use crate::{math::Ray, scene::material::Material};

use super::cast_result::CastResult;

pub trait Shape {
    fn intersect(&self, ray: Ray) -> Option<CastResult>;
    fn material(&self) -> &Material;
}