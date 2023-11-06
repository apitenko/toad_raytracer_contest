use std::f32::consts::{PI, TAU};

use crate::{
    math::{Ray, RayBounce, Vec3},
    scene::material::MaterialShared,
};

use super::{
    cast_result::{CastIntersectionResult, CastResult},
    shape::Shape,
    uv_set::UVSet,
};

#[derive(Clone, Debug)]
pub struct Triangle {
    pub material: MaterialShared,
    pub vertices: [Vec3; 3],
    pub uv: UVSet,
    pub normals: [Vec3; 3],
    pub tangents: [Vec3; 3],
    pub bitangents: [Vec3; 3],
}

impl Triangle {
    pub fn calculate_geometry_normal(&self) -> Vec3 {
        
        let edge1 = self.vertices[1] - self.vertices[0];
        let edge2 = self.vertices[2] - self.vertices[0];
        return Vec3::cross(edge1, edge2).normalized();
    }
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
    fn intersect(&self, ray: Ray, inside: bool) -> Option<CastIntersectionResult> {
        let vertex0: Vec3 = self.vertices[0];
        let vertex1: Vec3 = self.vertices[1];
        let vertex2: Vec3 = self.vertices[2];
        let edge1 = vertex1 - vertex0;
        let edge2 = vertex2 - vertex0;
        let h = Vec3::cross(ray.direction(), edge2);
        let a = Vec3::dot(edge1, h);

        if (a > -f32::EPSILON) & (a < f32::EPSILON) {
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

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * Vec3::dot(edge2, q);
        const EPSILON_OR_SOMETHING:f32 = 0.001;
        if t > f32::EPSILON {
            let intersection_point = ray.point_at_parameter(t);// - EPSILON_OR_SOMETHING * ray.direction();
            return Some(CastIntersectionResult {
                intersection_point,
                distance_traversed: t,//(ray.direction() * t).length(),
                raw_uvw: [u, v, w],
                triangle: self as *const Triangle,
                front_face: a > 0.0
            });
        } else {
            return None;
        }
    }
}
