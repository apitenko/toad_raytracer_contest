use crate::{primitives::{triangle::Triangle, cast_result::CastResult}, math::Ray};

pub trait AccelerationStructure {
    fn push_triangle(&mut self, insert_triangle: Triangle);
    fn single_cast(&self, ray: Ray, inside: bool) -> CastResult;
}   