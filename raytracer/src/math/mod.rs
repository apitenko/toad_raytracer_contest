use core::arch::x86_64::*;
use std::{
    fmt::{write, Debug},
    intrinsics::fabsf32,
    mem::MaybeUninit,
    u128,
};


pub mod random;
pub mod vec3;
pub mod mat44;
pub mod ray;

pub use vec3::Vec3;
pub use mat44::Mat44;
pub use ray::Ray;
pub use ray::RayBounce;

pub trait Saturatable {
    fn saturate(&self) -> f32;
}

impl Saturatable for f32 {
    fn saturate(&self) -> f32 {
        self.clamp(f32::EPSILON, 1.0 - f32::EPSILON)
    }
}