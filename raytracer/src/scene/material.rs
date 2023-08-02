use std::{
    mem::{zeroed, MaybeUninit},
    sync::Arc,
};

use crate::{
    math::{random::random_in_unit_sphere, refract, Ray, Vec3},
    primitives::cast_result::CastResult,
};

use super::texture::TextureShared;

pub struct Material {
    pub color: Vec3,
    pub specular_power: f32,
    pub texture: TextureShared,
}

impl Material {
    pub const fn new(color: Vec3, specular_power: f32, texture: TextureShared) -> Self {
        Self {
            color,
            specular_power,
            texture,
        }
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

    const fn invalid_mat() -> Self {
        unsafe {
            Self {
                mat: MaybeUninit::zeroed().assume_init(),
            }
        }
    }

    pub const INVALID_MAT: Self = Self::invalid_mat();
}
