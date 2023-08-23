use crate::{
    math::Vec3,
    primitives::{shape::Shape, sphere::Sphere, skybox::Skybox, mesh::Mesh},
};

use super::{acceleration_structure::SVORoot, lights::light::Light, texture::TextureShared, material::MaterialStorage, camera::Camera};

pub struct Scene {
    pub camera: Camera,
    pub geometry: SVORoot,
    pub lights: Vec<Box<dyn Light>>,
    pub skybox: Skybox,
    pub material_storage: MaterialStorage
}

impl Scene {
    pub fn new(skybox_texture: TextureShared) -> Self {
        Self {
            camera: Camera::new(),
            geometry: SVORoot::empty(),
            lights: Vec::new(),
            skybox: Skybox::new(skybox_texture),
            material_storage: MaterialStorage::new()
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.geometry.add_mesh(mesh);
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
