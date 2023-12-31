use std::path::Path;

use crate::constants::DEFAULT_IOR;
use crate::math::Vec3;
use crate::primitives::skybox::Skybox;
use crate::scene::acceleration_structure::acceleration_structure::AccelerationStructure;
use crate::{constants::DEFAULT_ASPECT_RATIO, primitives::triangle::Triangle};

use super::acceleration_structure::AccelerationStructureType;

use super::material::{IMaterialStorage, Material, MaterialShared};
use super::texture::sampler::Sampler;
use super::texture::texture::Texture;
use super::texture::texture_transform::TextureTransform;
use super::{camera::Camera, lights::light::Light, material::MaterialStorage};

pub struct Scene {
    pub camera: Camera,
    pub geometry: AccelerationStructureType,
    pub lights: Vec<Box<dyn Light>>,
    pub skybox: Skybox,
    pub material_storage: MaterialStorage,
    pub aspect_ratio: f32,
    pub default_material: MaterialShared,
}

impl Scene {
    pub fn new() -> anyhow::Result<Self> {
        let mut material_storage = MaterialStorage::new();

        // let default_texture = material_storage.push_texture(Texture::make_default_texture()?);
        let default_sampler = Sampler::new(
            &mut material_storage,
            Texture::make_default_texture()?,
            super::texture::sampler::MinFilter::Nearest,
            super::texture::sampler::MagFilter::Nearest,
            0,
            TextureTransform::default(),
        );

        let default_normal_sampler = Sampler::new(
            &mut material_storage,
            Texture::make_default_normal_map()?,
            super::texture::sampler::MinFilter::Nearest,
            super::texture::sampler::MagFilter::Nearest,
            0,
            TextureTransform::default(),
        );

        let default_material = material_storage.push_material(Material {
            color_factor: Vec3::ONE,
            ior: DEFAULT_IOR,
            emission_factor: Vec3::ONE,
            roughness_factor: 0.80,
            metallic_factor: 0.00,
            color_texture: default_sampler.clone(),
            metallic_roughness_texture: default_sampler.clone(),
            emission_texture: default_sampler.clone(),
            normal_texture: default_normal_sampler.clone(),
            transmission_factor: 0.0,
            transmission_texture: default_sampler.clone(),
            double_sided: true, // TODO: KHR_materials_volume .doubleSided property
        });
        // let skybox_texture =  material_storage.push_texture(Texture::new_from_file(&Path::new("./res/skybox.png"))?);
        let skybox_texture =  material_storage.push_texture(Texture::new_from_raw_bytes(TEXTURE_EMBEDDED_SKYBOX)?);

        Ok(Self {
            camera: Camera::new(),
            geometry: AccelerationStructureType::empty(),
            lights: Vec::new(),
            skybox: Skybox::new(skybox_texture),
            material_storage,
            aspect_ratio: DEFAULT_ASPECT_RATIO,
            default_material,
        })
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn push_triangle(&mut self, tri: Triangle) {
        self.geometry.push_triangle(tri);
    }
}

#[derive(Clone)]
pub struct TotallySafeSceneWrapper(*const Scene);

impl TotallySafeSceneWrapper {
    pub fn new(scene: *const Scene) -> Self {
        Self(scene)
    }

    pub fn get(&self) -> *const Scene {
        self.0
    }
}

unsafe impl Send for TotallySafeSceneWrapper {}
unsafe impl Sync for TotallySafeSceneWrapper {}

static TEXTURE_EMBEDDED_SKYBOX: &[u8] = include_bytes!("skybox.png");
