use std::path::Path;

use crate::{
    math::Vec3,
    primitives::{mesh::Mesh, shape::Shape, skybox::Skybox, sphere::Sphere, triangle::Triangle}, constants::DEFAULT_ASPECT_RATIO,
};

use super::{
    acceleration_structure::OctreeRoot,
    camera::Camera,
    lights::light::Light,
    material::MaterialStorage,
    texture::{Texture, TextureShared},
};

pub struct Scene {
    pub camera: Camera,
    pub geometry: OctreeRoot,
    pub lights: Vec<Box<dyn Light>>,
    // pub skybox: Skybox,
    pub material_storage: MaterialStorage,
    pub aspect_ratio: f32,
}

impl Scene {
    pub fn new() -> anyhow::Result<Self> {
        let mut material_storage = MaterialStorage::new();
        // let skybox_texture =  material_storage.push_texture(Texture::new_from_file(&Path::new("./res/skybox.png"))?);

        Ok(Self {
            camera: Camera::new(),
            geometry: OctreeRoot::empty(),
            lights: Vec::new(),
            // skybox: Skybox::new(skybox_texture),
            material_storage,
            aspect_ratio: DEFAULT_ASPECT_RATIO
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
