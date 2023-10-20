use crate::scene::texture::samplable::Samplable;
use crate::{math::Vec3, util::unresizable_array::UnresizableArray};
use std::{
    mem::{transmute, zeroed, MaybeUninit},
    sync::Arc,
};

use super::texture::{sampler::Sampler, texture::Texture, texture::TextureShared};

#[derive(Clone)]
pub struct Material {
    // ? PBR stuff:
    pub color_factor: Vec3,
    pub color_texture: Sampler,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub metallic_roughness_texture: Sampler,

    // ? glass
    pub ior: f32,

    // ? other
    pub emission_factor: Vec3,
    pub emission_texture: Sampler,
    pub normal_texture: Sampler,

    pub transmission_factor: f32,
    pub transmission_texture: Sampler,
    // pub subsurface: f32,
    // pub anisotropic: f32,
    // pub sheen: f32,
    // pub sheen_tint: f32,
    // pub clearcoat: f32,
    // pub clearcoat_gloss: f32,
}

type MaterialStorageForDefault = MaterialStorageSized<6, 6>;

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
    // static ref MATERIAL_DATA_DEFAULT: Box<Material> = {
    //     Box::new(Material {
    //         ..Default::default()
    //     })
    // };
    // pub static ref MATERIAL_DEFAULT: MaterialShared = {
    //     MaterialShared::new(MATERIAL_DATA_DEFAULT.as_ref() as *const Material)
    // };

    // static ref MATERIAL_STORAGE_FOR_DEFAULTS: Box<dyn IMaterialStorage> = {
    //     Box::new(MaterialStorageForDefault::new())
    // };

    // pub static ref SAMPLER_DEFAULT: Sampler = unsafe {
    //     let im_shooting_myself_in_the_leg =
    //     MATERIAL_STORAGE_FOR_DEFAULTS.as_ref();


    //     Sampler::new(
    //         im_shooting_myself_in_the_leg,
    //         TEXTURE_DATA_DEFAULT.as_ref().clone(),
    //         crate::scene::texture::sampler::MinFilter::Nearest,
    //         crate::scene::texture::sampler::MagFilter::Nearest,
    //     )
    // };

}

// impl Default for Material {
//     fn default() -> Self {
//         Self {
//             uv_scale: 1.0,
//             color_factor: Vec3::ONE,
//             fresnel_coefficient: 4.0,
//             emission_color: Vec3::ONE,
//             emission_power: 0.0,
//             specular: 0.00 * Vec3::ONE,
//             roughness: 1.00,
//             color_albedo: SAMPLER_DEFAULT.clone(),
//         }
//     }
// }

impl Material {
    // pub fn empty() -> Self {
    //     Self {
    //         color_albedo: TextureShared::uninitialized(),
    //         ..Default::default()
    //     }
    // }

    #[inline]
    fn sample_uv_scaled(&self, texture: &Sampler, uv: &[(f32, f32); 4], mip: f32) -> Vec3 {
        let material_albedo = texture.sample(uv, mip);
        return material_albedo;
    }

    #[inline]
    pub fn sample_albedo(&self, uv: &[(f32, f32); 4], mip: f32) -> Vec3 {
        // TEXTURE_DATA_DEFAULT.sample(uv.0, uv.1) * self.color_tint
        self.sample_uv_scaled(&self.color_texture, uv, mip) * self.color_factor
    }

    #[inline]
    pub fn sample_roughness_metallic(&self, uv: &[(f32, f32); 4], mip: f32) -> (f32,f32) {
        let sample = self.sample_uv_scaled(&self.metallic_roughness_texture, uv, mip);
        (sample.y(), sample.z())
    }

    #[inline]
    pub fn sample_metallic(&self, uv: &[(f32, f32); 4], mip: f32) -> f32 {
        let sample = self.sample_uv_scaled(&self.metallic_roughness_texture, uv, mip);
        sample.z() * self.metallic_factor
    }

    #[inline]
    pub fn sample_roughness(&self, uv: &[(f32, f32); 4], mip: f32) -> f32 {
        self.sample_uv_scaled(&self.metallic_roughness_texture, uv, mip).y() * self.roughness_factor
    }

    #[inline]
    pub fn sample_emission(&self, uv: &[(f32, f32); 4], mip: f32) -> Vec3 {
        self.sample_uv_scaled(&self.emission_texture, uv, mip) * self.emission_factor
    }

    #[inline]
    pub fn sample_normal(&self, uv: &[(f32, f32); 4], mip: f32) -> Vec3 {
        self.sample_uv_scaled(&self.normal_texture, uv, mip).normalized()
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

pub trait IMaterialStorage {
    fn push_material(&mut self, mat: Material) -> MaterialShared;
    fn push_texture(&mut self, tex: Texture) -> TextureShared;
}

pub struct MaterialStorageSized<const MATERIALS_MAX: usize, const TEXTURES_MAX: usize> {
    materials: UnresizableArray<Material, { MATERIALS_MAX }>,
    textures: UnresizableArray<Texture, { TEXTURES_MAX }>,
}

pub type MaterialStorage = MaterialStorageSized<600, 600>;

impl<const MATERIALS_MAX: usize, const TEXTURES_MAX: usize>
    MaterialStorageSized<{ MATERIALS_MAX }, { TEXTURES_MAX }>
{
    pub fn new() -> Self {
        Self {
            materials: UnresizableArray::<Material, { MATERIALS_MAX }>::with_capacity(),
            textures: UnresizableArray::<Texture, { TEXTURES_MAX }>::with_capacity(),
        }
    }
}

impl<const MATERIALS_MAX: usize, const TEXTURES_MAX: usize> IMaterialStorage
    for MaterialStorageSized<{ MATERIALS_MAX }, { TEXTURES_MAX }>
{
    fn push_material(&mut self, mat: Material) -> MaterialShared {
        let ptr = self.materials.push(mat);
        MaterialShared::new(ptr)
    }

    fn push_texture(&mut self, tex: Texture) -> TextureShared {
        let ptr = self.textures.push(tex);
        TextureShared::new(ptr)
    }
}

unsafe impl<const MATERIALS_MAX: usize, const TEXTURES_MAX: usize> Send
    for MaterialStorageSized<{ MATERIALS_MAX }, { TEXTURES_MAX }>
{
}
unsafe impl<const MATERIALS_MAX: usize, const TEXTURES_MAX: usize> Sync
    for MaterialStorageSized<{ MATERIALS_MAX }, { TEXTURES_MAX }>
{
}
