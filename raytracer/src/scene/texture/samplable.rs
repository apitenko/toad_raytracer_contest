use crate::math::Vec3;

pub trait Samplable {    
    fn sample(&self, uv: &[(f32, f32); 4], mip: f32) -> Vec3;
}