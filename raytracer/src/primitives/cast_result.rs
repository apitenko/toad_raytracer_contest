use std::{mem::MaybeUninit, ptr::null};

use crate::{
    constants::COLOR_CALL_PARAMETERS,
    math::{Ray, Vec3},
    scene::material::MaterialShared,
};

use super::{triangle::Triangle, uv_set::{UVSet, UVChannel}};

pub struct CastIntersectionResult {
    pub distance_traversed: f32,
    pub intersection_point: Vec3,
    pub raw_uvw: [f32; 3],
    pub triangle: *const Triangle,
    pub front_face: bool,
}

impl CastIntersectionResult {
    pub const MISS: Self = Self {
        intersection_point: Vec3::ZERO,
        distance_traversed: f32::INFINITY,
        triangle: null(),
        raw_uvw: [0.0, 0.0, 0.0],
        front_face: false
    };

    #[inline]
    pub fn has_missed(&self) -> bool {
        return self.distance_traversed == f32::INFINITY;
    }
}

pub struct CastResult {
    pub distance_traversed: f32,
    pub intersection_point: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv_color: [(f32, f32); 4],
    pub uv_metalrough: [(f32, f32); 4],
    pub uv_normalmap: [(f32, f32); 4],
    pub uv_emission: [(f32, f32); 4],
    pub uv_transmission: [(f32, f32); 4],
    pub material: MaterialShared,
}

impl CastResult {
    #[inline]
    pub fn has_missed(&self) -> bool {
        return self.distance_traversed == f32::INFINITY;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ConeCastResult {
    pub accumulated_color: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct ConeCastResultStep {
    pub accumulated_color: Vec3,
    pub accumulated_density: f32, // see nvidia vxgi
}

impl ConeCastResultStep {
    pub fn empty() -> Self {
        Self {
            accumulated_color: Vec3::ZERO,
            accumulated_density: 0.0,
        }
    }

    pub fn merge(left: Self, right: Self) -> Self {
        Self {
            accumulated_color: left.accumulated_color + right.accumulated_color,
            accumulated_density: left.accumulated_density + right.accumulated_density,
        }
    }
}

#[deprecated]
fn skybox_color(ray: &Ray) {
    // "skybox"
    let ray_normalized = ray.direction().normalized();
    let t = 0.5 * (ray_normalized.y() + 1.0);
    (1.0 - t) * Vec3::ONE + t * COLOR_CALL_PARAMETERS;
}

impl CastIntersectionResult {
    pub fn resolve(&self) -> Option<CastResult> {
        // debug_assert!(!self.triangle.is_null());
        if self.triangle.is_null() {
            return None;
        }

        let triangle = unsafe { &*self.triangle };
        let [u, v, w] = self.raw_uvw;

        let uv_color = interpolate_uvs([w, u, v], &triangle.uv.channels_color);
        let uv_metalrough = interpolate_uvs([w, u, v], &triangle.uv.channels_metalrough);
        let uv_normalmap = interpolate_uvs([w, u, v], &triangle.uv.channels_normalmap);
        let uv_emission = interpolate_uvs([w, u, v], &triangle.uv.channels_emission);
        let uv_transmission = interpolate_uvs([w, u, v], &triangle.uv.channels_transmission);

        let mut normal = interpolate_normals([w, u, v], triangle.normals);
        let mut tangent = interpolate_normals([w, u, v], triangle.tangents);
        let mut bitangent = interpolate_normals([w, u, v], triangle.bitangents);

        if !self.front_face && triangle.material.get().double_sided {
            normal = -normal;
            tangent = -tangent;
            bitangent = -bitangent;
        }

        return Some(CastResult {
            intersection_point: self.intersection_point,
            distance_traversed: self.distance_traversed,
            normal,
            tangent,
            bitangent,
            uv_color,
            uv_metalrough,
            uv_normalmap,
            uv_emission,
            uv_transmission,
            material: triangle.material.clone(),
            // triangle: self.clone()
        });
    }
}

#[inline]
fn interpolate_uvs(intersection_wuv: [f32; 3], self_uv_channels: &[UVChannel; 4]) -> [(f32, f32); 4] {
    [
        interpolate_uv(intersection_wuv, &self_uv_channels[0].points),
        interpolate_uv(intersection_wuv, &self_uv_channels[1].points),
        interpolate_uv(intersection_wuv, &self_uv_channels[2].points),
        interpolate_uv(intersection_wuv, &self_uv_channels[3].points),
    ]
}

#[inline]
fn interpolate_uv(wuv: [f32; 3], triangle_uv: &[[f32; 2]; 3]) -> (f32, f32) {
    add_f32([
        mul_f32(triangle_uv[0], wuv[0]),
        mul_f32(triangle_uv[1], wuv[1]),
        mul_f32(triangle_uv[2], wuv[2]),
    ])
}

#[inline]
fn interpolate_normals(wuv: [f32; 3], normals: [Vec3; 3]) -> Vec3 {
    wuv[0] * normals[0] + wuv[1] * normals[1] + wuv[2] * normals[2]
}

#[inline]
fn mul_f32(triangle_uv: [f32; 2], m: f32) -> [f32; 2] {
    [triangle_uv[0] * m, triangle_uv[1] * m]
}

#[inline]
fn add_f32(fgsfds: [[f32; 2]; 3]) -> (f32, f32) {
    (
        fgsfds[0][0] + fgsfds[1][0] + fgsfds[2][0],
        fgsfds[0][1] + fgsfds[1][1] + fgsfds[2][1],
    )
}
