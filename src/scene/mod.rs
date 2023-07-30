use crate::{primitives::Camera, math::Vec3};

pub mod workload;

pub struct VoxelizedScene {}

pub struct Scene {
    pub camera: Camera,
    pub lower_left_corner: Vec3,
    pub width_in_units: Vec3,
    pub height_in_units: Vec3,
    pub origin: Vec3,
    // voxelized_scene: VoxelizedScene
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(Vec3::ZERO),
            lower_left_corner: Vec3::new([-2.0, -1.0, -1.0]),
            width_in_units: Vec3::new([4.0, 0.0, 0.0]),
            height_in_units: Vec3::new([0.0, 2.0, 0.0]),
            origin: Vec3::ZERO
        }
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