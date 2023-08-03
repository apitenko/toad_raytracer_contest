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
    pub color_tint: Vec3, // non-PBR parameter; use Vec3::ONE to disable it
    pub fresnel_coefficient: f32,
    // pub subsurface: f32,
    // pub metallic: f32,
    pub specular: f32,
    // pub specular_tint: f32,
    // pub roughness: f32,
    // pub anisotropic: f32,
    // pub sheen: f32,
    // pub sheen_tint: f32,
    // pub clearcoat: f32,
    // pub clearcoat_gloss: f32,
    pub albedo: TextureShared,
}

impl Material {}

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


// pub struct DefaultMaterialsMap {

// }

// pub fn default_materials_map() {}