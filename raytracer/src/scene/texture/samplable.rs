use crate::math::Vec3;

pub trait Samplable {    
    fn sample(&self, u: f32, v: f32, mip: f32) -> Vec3;
}