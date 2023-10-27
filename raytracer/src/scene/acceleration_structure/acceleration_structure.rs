use crate::{
    math::{cone::Cone, Ray},
    primitives::{
        cast_result::{CastIntersectionResult, ConeCastResult},
        triangle::Triangle,
    },
};

pub trait AccelerationStructure {
    fn push_triangle(&mut self, insert_triangle: Triangle);
    fn single_cast(&self, ray: Ray, inside: bool) -> CastIntersectionResult;
    fn cone_cast(&self, cone: Cone) -> ConeCastResult;
    fn inject_emittance_data(&mut self, ray: Ray);
}
