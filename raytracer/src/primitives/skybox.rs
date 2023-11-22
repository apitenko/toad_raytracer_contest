use std::f32::consts::PI;

use crate::{constants::MISS_COLOR_VEC3, math::Vec3, scene::texture::texture::TextureShared};

pub const SKYBOX_EMISSION_INTENSITY: f32 = 0.001;
pub const SKYBOX_MISS_INTENSITY: f32 = 1.0;

pub struct Skybox {
    texture: TextureShared,
}

fn faceuv_to_texture_uv(face: u32, u: f32, v: f32) -> (f32, f32) { // u,v

    debug_assert!(face <= 5);

    let (start_u, start_v) = match face {
        0 => (0.5, 0.33333333),
        1 => (0.0, 0.33333333),
        2 => (0.25, 0.0),
        3 => (0.25, 0.6666666),
        4 => (0.25, 0.3333333),
        5 => (0.75, 0.3333333),
        _ => panic!("cubemap face is >5")
    };

    let (width_uv, height_uv) = (0.25, 0.25);

    return (start_u + width_uv * u, start_v + height_uv * v);
}

fn convert_xyz_to_cube_uv(direction: Vec3) -> (u32, f32, f32) // face, u, v
{
    let dir_abs = direction.abs();

    let isXPositive: bool = if direction.x() > 0.0 { true } else { false };
    let isYPositive: bool = if direction.y() > 0.0 { true } else { false };
    let isZPositive: bool = if direction.z() > 0.0 { true } else { false };

    let mut maxAxis = 0.0;
    let mut uc = 0.0;
    let mut vc = 0.0;
    let mut index = 0;

    // POSITIVE X
    if isXPositive && dir_abs.x() >= dir_abs.y() && dir_abs.x() >= dir_abs.z() {
        // u (0 to 1) goes from +z to -z
        // v (0 to 1) goes from -y to +y
        maxAxis = dir_abs.x();
        uc = -direction.z();
        vc = direction.y();
        index = 0;
    }
    // NEGATIVE X
    if !isXPositive && dir_abs.x() >= dir_abs.y() && dir_abs.x() >= dir_abs.z() {
        // u (0 to 1) goes from -z to +z
        // v (0 to 1) goes from -y to +y
        maxAxis = dir_abs.x();
        uc = direction.z();
        vc = direction.y();
        index = 1;
    }
    // POSITIVE Y
    if isYPositive && dir_abs.y() >= dir_abs.x() && dir_abs.y() >= dir_abs.z() {
        // u (0 to 1) goes from -x to +x
        // v (0 to 1) goes from +z to -z
        maxAxis = dir_abs.y();
        uc = direction.x();
        vc = -direction.z();
        index = 2;
    }
    // NEGATIVE Y
    if !isYPositive && dir_abs.y() >= dir_abs.x() && dir_abs.y() >= dir_abs.z() {
        // u (0 to 1) goes from -x to +x
        // v (0 to 1) goes from -z to +z
        maxAxis = dir_abs.y();
        uc = direction.x();
        vc = direction.z();
        index = 3;
    }
    // POSITIVE Z
    if isZPositive && dir_abs.z() >= dir_abs.x() && dir_abs.z() >= dir_abs.y() {
        // u (0 to 1) goes from -x to +x
        // v (0 to 1) goes from -y to +y
        maxAxis = dir_abs.z();
        uc = direction.x();
        vc = direction.y();
        index = 4;
    }
    // NEGATIVE Z
    if !isZPositive && dir_abs.z() >= dir_abs.x() && dir_abs.z() >= dir_abs.y() {
        // u (0 to 1) goes from +x to -x
        // v (0 to 1) goes from -y to +y
        maxAxis = dir_abs.z();
        uc = -direction.x();
        vc = direction.y();
        index = 5;
    }

    // Convert range from -1 to 1 to 0 to 1
    let u = 0.5 * (uc / maxAxis + 1.0);
    let v = 0.5 * (vc / maxAxis + 1.0);

    return (index, u, v);
}

impl Skybox {
    // fn uv(direction: Vec3) -> (f32, f32) {
    //     let theta = f32::acos(direction.y()) / -PI;
    //     let phi = f32::atan2(direction.x(), -direction.z()) / -PI * 0.5;
    //     return (phi, theta);
    // }

    // fn sample(&self, u: f32, v: f32) -> Vec3 {
    //     let sample = self.texture.get().sample(u, v);
    //     sample
    // }

    pub fn sample_from_direction(&self, direction: Vec3) -> Vec3 {
        let (face, u, v) = convert_xyz_to_cube_uv(direction);
        let (u, v) = faceuv_to_texture_uv(face, u, v);
        self.texture.get().sample(u, v)
    }

    pub fn new(texture: TextureShared) -> Self {
        Self { texture }
    }
}
