use std::f32::consts::{PI, TAU};

use crate::{
    math::{Ray, RayBounce, Vec3},
    scene::material::MaterialShared,
};

use super::{cast_result::CastResult, shape::Shape};

pub struct Triangle {
    // pub position: Vec3,
    pub material: MaterialShared,
    pub vertices: [Vec3; 3],
}

impl Triangle {
    pub fn new(position: Vec3, vertices: [Vec3; 3], material: MaterialShared) -> Self {
        Self { material, vertices: [
            position + vertices[0],
            position + vertices[1],
            position + vertices[2],
        ] }
    }
}

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

        if (v < 0.0 || u + v > 1.0) {
            return None;
        }

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = f * Vec3::dot(edge2, q);

        if t > EPSILON
        // ray intersection
        {
            let normal = Vec3::cross(edge1, edge2).normalized();
            let intersection_point = ray.point_at_parameter(t);
            return Some(CastResult {
                intersection_point,
                distance_traversed: (ray.direction() * t).length(),
                normal,
                material: self.material.clone(),
                uv: (u, v)
            });
        } else {
            // This means that there is a line intersection but not a ray intersection.
            return None;
        }
    }
}
