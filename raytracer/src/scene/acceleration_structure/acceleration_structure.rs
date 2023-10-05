use crate::{primitives::{triangle::Triangle, cast_result::{CastResult, ConeCastResult}}, math::{Ray, cone::Cone}};

pub trait AccelerationStructure {
    fn push_triangle(&mut self, insert_triangle: Triangle);
    fn single_cast(&self, ray: Ray, inside: bool) -> CastResult;
    fn cone_cast(&self, cone: Cone) -> ConeCastResult;
    fn inject_emittance_data(&mut self, ray: Ray);
}   