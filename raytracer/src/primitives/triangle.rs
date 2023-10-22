use std::f32::consts::{PI, TAU};

use crate::{
    math::{Ray, RayBounce, Vec3},
    scene::material::MaterialShared,
};

use super::{cast_result::CastResult, shape::Shape, uv_set::UVSet};

#[derive(Clone, Debug)]
pub struct Triangle {
    pub material: MaterialShared,
    pub vertices: [Vec3; 3],
    pub uv: UVSet,
    pub normals: [Vec3; 3],
    pub tangents: [Vec3; 3],
    pub bitangents: [Vec3; 3],
}

// impl Default for Triangle {
//     fn default() -> Self {
//         Self {
//             vertices: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
//             uv: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
//             normals: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO]
//         }
//     }
// }

// impl Triangle {
//     pub fn from_vertices(p0: Vec3, p1: Vec3, p2: Vec3, material: MaterialShared) -> Self {
//         let normal = Vec3::calculate_normal_from_points(p0, p1, p2);
//         Self {
//             vertices: [p0, p1, p2],
//             uv: UVSet::empty(),
//             normals: [normal, normal, normal],
//             material,
//         }
//     }
// }

impl Shape for Triangle {
    fn intersect(&self, ray: Ray, inside: bool) -> Option<CastResult> {
        const EPSILON: f32 = 0.0000001;
        let vertex0: Vec3 = self.vertices[0];
        let vertex1: Vec3 = self.vertices[1];
        let vertex2: Vec3 = self.vertices[2];
        let edge1 = vertex1 - vertex0;
        let edge2 = vertex2 - vertex0;
        let h = Vec3::cross(ray.direction(), edge2);
        let a = Vec3::dot(edge1, h);

        if a > -EPSILON && a < EPSILON {
            return None; // This ray is parallel to this triangle.
        }

        let f = 1.0 / a;
        let s = ray.origin() - vertex0;
        let u = f * Vec3::dot(s, h);

        if (u < 0.0 || u > 1.0) {
            return None;
        }

        let q = Vec3::cross(s, edge1);
        let v = f * Vec3::dot(ray.direction(), q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let w = 1.0 - u - v;

        let uv = interpolate_uvs([w, u, v], &self.uv);

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * Vec3::dot(edge2, q);

        if t > EPSILON
        // ray intersection
        {
            // let geometry_normal = Vec3::cross(edge1, edge2).normalized();
            let normal = interpolate_normals([w, u, v], self.normals);
            let tangent = interpolate_normals([w, u, v], self.tangents);
            let bitangent = interpolate_normals([w, u, v], self.bitangents);
            // let normal = geometry_normal;//* interpolated_normal;

            let intersection_point = ray.point_at_parameter(t);
            return Some(CastResult {
                intersection_point,
                distance_traversed: (ray.direction() * t).length(),
                normal,
                tangent,
                bitangent,
                uv,
                material: self.material.clone(),
                // triangle: self.clone()
            });
        } else {
            // This means that there is a line intersection but not a ray intersection.
            return None;
        }
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
fn interpolate_normals(uvw: [f32; 3], normals: [Vec3; 3]) -> Vec3 {
    uvw[0] * normals[0] + uvw[1] * normals[1] + uvw[2] * normals[2]
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
