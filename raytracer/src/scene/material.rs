use std::sync::Arc;

use crate::{
    math::{random::random_in_unit_sphere, refract, Ray, Vec3},
    primitives::cast_result::CastResult,
};

pub struct Material {
    pub color: Vec3,
    pub specular_power: f32,
}

impl Material {
    pub const fn new(color: Vec3, specular_power: f32) -> Self {
        Self { color, specular_power }
    }
}

#[derive(Clone)]
pub struct MaterialShared {
    mat: *const Material,
}

unsafe impl Send for MaterialShared {}
unsafe impl Sync for MaterialShared {}

impl MaterialShared {
    pub fn new(mat: *const Material) -> Self {
        Self { mat }
    }

    pub fn get(&self) -> &Material {
        unsafe {
            return &*self.mat as &Material;
        }
    }

    const DEFAULT_MAT_IMPL: Material = Material::new(Vec3::ONE, 0.5);
    pub const DEFAULT_MAT: Self = Self {
        mat: &Self::DEFAULT_MAT_IMPL,
    };
}
