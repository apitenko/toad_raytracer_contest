use crate::{math::Vec3, primitives::cast_result::CastResult};

pub trait Light {
    fn get_emission(&self, at_point: Vec3) -> Vec3;
    fn normal_from(&self, origin: Vec3) -> (f32, Vec3);
}