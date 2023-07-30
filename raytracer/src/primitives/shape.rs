use crate::math::Ray;

use super::cast_result::CastResult;

pub trait Shape {
    fn intersect(&self, ray: Ray) -> Option<CastResult>;
}