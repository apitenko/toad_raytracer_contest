use crate::{
    math::Vec3,
    primitives::{camera::Camera, shape::Shape, sphere::Sphere},
};

use super::svo::SVORoot;

pub struct Scene {
    pub camera: Camera,
    pub lower_left_corner: Vec3,
    pub width_in_units: Vec3,
    pub height_in_units: Vec3,
    pub origin: Vec3,

    pub geometry: SVORoot,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(Vec3::ZERO),
            lower_left_corner: Vec3::new([-2.0, -1.0, -1.0]),
            width_in_units: Vec3::new([4.0, 0.0, 0.0]),
            height_in_units: Vec3::new([0.0, 2.0, 0.0]),
            origin: Vec3::ZERO,
            geometry: SVORoot::empty(),
        }
    }

    pub fn push_shape(&mut self, shape: *const dyn Shape) {
        self.geometry.push_shape(shape);
    }

    pub fn get_object(&self) -> Sphere {
        return Sphere::new(Vec3::new([0.0, 0.0, -1.0]), 0.5);
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
