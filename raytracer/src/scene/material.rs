use std::{
    mem::{zeroed, MaybeUninit},
    sync::Arc,
};

use crate::{math::Vec3, scene::texture::Texture, util::unresizable_array::UnresizableArray};

use super::texture::TextureShared;

#[derive(Clone, Copy)]
pub struct Material {
    pub uv_scale: f32,
    pub color_factor: Vec3, // non-PBR parameter; use Vec3::ONE to disable it
    pub fresnel_coefficient: f32,
    pub emission_color: Vec3, // TODO: texture?
    pub emission_power: f32,
    // pub subsurface: f32,
    // pub metallic_factor: f32, // TODO: Metallic/Roughness workflow; unused at the moment
    // pub metallic: TextureShared,
    pub specular: Vec3, // TODO: texture
    // pub specular_tint: f32,
    pub roughness: f32, // TODO: texture
    // pub anisotropic: f32,
    // pub sheen: f32,
    // pub sheen_tint: f32,
    // pub clearcoat: f32,
    // pub clearcoat_gloss: f32,
    pub color_albedo: TextureShared,
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
    static ref MATERIAL_DATA_DEFAULT: Box<Material> = {
        Box::new(Material {
            ..Default::default()
        })
    };
    pub static ref MATERIAL_DEFAULT: MaterialShared = {
        MaterialShared::new(MATERIAL_DATA_DEFAULT.as_ref() as *const Material)
    };
}

impl Default for Material {
    fn default() -> Self {
        Self {
            uv_scale: 1.0,
            color_factor: Vec3::ONE,
            fresnel_coefficient: 4.0,
            emission_color: Vec3::ONE,
            emission_power: 0.0,
            specular: 0.00 * Vec3::ONE,
            roughness: 1.0,
            color_albedo: TEXTURE_DEFAULT.clone(),
        }
    }
}

impl Material {
    // pub fn empty() -> Self {
    //     Self {
    //         color_albedo: TextureShared::uninitialized(),
    //         ..Default::default()
    //     }
    // }

    fn sample_uv_scaled(&self, texture: &TextureShared, uv: (f32, f32)) -> Vec3 {
        let material_albedo = {
            let u = (uv.0 * self.uv_scale).fract();
            let v = (uv.1 * self.uv_scale).fract();
            texture.get().sample(u, v)
        };
        return material_albedo;
    }

    pub fn sample_albedo(&self, uv: (f32, f32)) -> Vec3 {
        // TEXTURE_DATA_DEFAULT.sample(uv.0, uv.1) * self.color_tint
        self.sample_uv_scaled(&self.color_albedo, uv) * self.color_factor
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

#[derive(Clone, Debug)]
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

    pub const fn null() -> Self {
        unsafe {
            Self {
                mat: std::ptr::null(),
            }
        }
    }

    pub fn valid(&self) -> bool {
        !self.mat.is_null()
    }

    // pub const INVALID_MAT: Self = Self::invalid_mat();
}

// pub struct DefaultMaterialsMap {

// }

// pub fn default_materials_map() {}

pub struct MaterialStorage {
    materials: UnresizableArray<Material, { Self::MATERIALS_MAX }>,
    textures: UnresizableArray<Texture, { Self::TEXTURES_MAX }>,
}

impl MaterialStorage {
    const MATERIALS_MAX: usize = 600;
    const TEXTURES_MAX: usize = 600;

    pub fn new() -> Self {
        Self {
            materials: UnresizableArray::<Material, { Self::MATERIALS_MAX }>::with_capacity(),
            textures: UnresizableArray::<Texture, { Self::TEXTURES_MAX }>::with_capacity(),
        }
    }

    pub fn push_material(&mut self, mat: Material) -> MaterialShared {
        let ptr = self.materials.push(mat);
        MaterialShared::new(ptr)
    }

    pub fn push_texture(&mut self, tex: Texture) -> TextureShared {
        let ptr = self.textures.push(tex);
        TextureShared::new(ptr)
    }
}

unsafe impl Send for MaterialStorage {}
unsafe impl Sync for MaterialStorage {}
