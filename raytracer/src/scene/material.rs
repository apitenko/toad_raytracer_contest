use std::{
    mem::{zeroed, MaybeUninit},
    sync::Arc,
};

use crate::{
    math::{random::random_in_unit_sphere, refract, Ray, Vec3},
    primitives::cast_result::CastResult,
    scene::texture::Texture,
};

use super::texture::TextureShared;

pub struct Material {
    pub uv_scale: f32,
    pub color_tint: Vec3, // non-PBR parameter; use Vec3::ONE to disable it
    pub fresnel_coefficient: f32,
    pub emission_color: Vec3, // TODO: texture?
    pub emission_power: f32,
    // pub subsurface: f32,
    // pub metallic: f32,
    pub specular: Vec3, // TODO: texture
    // pub specular_tint: f32,
    pub roughness: f32, // TODO: texture
    // pub anisotropic: f32,
    // pub sheen: f32,
    // pub sheen_tint: f32,
    // pub clearcoat: f32,
    // pub clearcoat_gloss: f32,
    pub albedo: TextureShared,
}

lazy_static::lazy_static! {
    static ref TEXTURE_DATA_DEFAULT: Box<Texture> = {
        let texture_default = Texture::make_default_texture()
            .expect("Material::Default -- make_default_texture failed");
        let texture_default = Box::new(texture_default);
        texture_default
    };
    static ref TEXTURE_DEFAULT: TextureShared = {
        TextureShared::new(TEXTURE_DATA_DEFAULT.as_ref() as *const Texture)
    };
}

impl Default for Material {
    fn default() -> Self {
        Self {
            uv_scale: 1.0,
            color_tint: Vec3::ONE,
            fresnel_coefficient: 4.0,
            emission_color: Vec3::ONE,
            emission_power: 0.0,
            specular: 0.3 * Vec3::ONE,
            roughness: 0.3,
            albedo: TEXTURE_DEFAULT.clone(),
        }
    }
}

impl Material {
    fn sample_uv_scaled(&self, texture: &TextureShared, uv: (f32, f32)) -> Vec3 {
        let material_albedo = {
            let material_color_tint = self.color_tint;
            let u = (uv.0 * self.uv_scale).fract();
            let v = (uv.1 * self.uv_scale).fract();
            material_color_tint * texture.get().sample(u, v)
        };
        return material_albedo;
    }

    pub fn sample_albedo(&self, uv: (f32, f32)) -> Vec3 {
        self.sample_uv_scaled(&self.albedo, uv) * self.color_tint
    }

    pub fn sample_roughness(&self, uv: (f32, f32)) -> f32 {
        return self.roughness;
    }

    pub fn sample_specular(&self, uv: (f32, f32)) -> Vec3 {
        return self.specular;
    }

    pub fn sample_emission(&self, uv: (f32, f32)) -> Vec3 {
        return self.emission_color * self.emission_power;
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

// pub struct DefaultMaterialsMap {

// }

// pub fn default_materials_map() {}
