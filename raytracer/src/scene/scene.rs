use crate::{
    math::Vec3,
    primitives::{camera::Camera, shape::Shape, sphere::Sphere},
};

use super::{svo::SVORoot, lights::light::Light};

pub struct Scene {
    pub camera: Camera,
    pub geometry: SVORoot,
    pub lights: Vec<Box<dyn Light>>
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            geometry: SVORoot::empty(),
            lights: Vec::new(),
        }
    }

    pub fn push_shape(&mut self, shape: *const dyn Shape) {
        self.geometry.push_shape(shape);
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
