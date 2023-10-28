use std::{mem::MaybeUninit, ptr::null};

use crate::{
    constants::COLOR_CALL_PARAMETERS,
    math::{Ray, Vec3},
    scene::material::MaterialShared,
};

use super::{triangle::Triangle, uv_set::UVSet};

pub struct CastIntersectionResult {
    pub distance_traversed: f32,
    pub intersection_point: Vec3,
    pub raw_uvw: [f32; 3],
    pub triangle: *const Triangle,
}

impl CastIntersectionResult {
    pub const MISS: Self = Self {
        intersection_point: Vec3::ZERO,
        distance_traversed: f32::INFINITY,
        triangle: null(),
        raw_uvw: [0.0, 0.0, 0.0],
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
    pub uv: [(f32, f32); 4],
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
        let uv = interpolate_uvs([w, u, v], &triangle.uv);

        // let geometry_normal = Vec3::cross(edge1, edge2).normalized();
        let normal = interpolate_normals([w, u, v], triangle.normals);
        let tangent = interpolate_normals([w, u, v], triangle.tangents);
        let bitangent = interpolate_normals([w, u, v], triangle.bitangents);
        // let normal = geometry_normal;//* interpolated_normal;

        return Some(CastResult {
            intersection_point: self.intersection_point,
            distance_traversed: self.distance_traversed,
            normal,
            tangent,
            bitangent,
            uv,
            material: triangle.material.clone(),
            // triangle: self.clone()
        });
    }
}

#[inline]
fn interpolate_uvs(intersection_wuv: [f32; 3], self_uv: &UVSet) -> [(f32, f32); 4] {
    [
        interpolate_uv(intersection_wuv, &self_uv.channels[0].points),
        interpolate_uv(intersection_wuv, &self_uv.channels[1].points),
        interpolate_uv(intersection_wuv, &self_uv.channels[2].points),
        interpolate_uv(intersection_wuv, &self_uv.channels[3].points),
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
