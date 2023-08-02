use crate::{math::Vec3, scene::material::MaterialShared};

use super::{cast_result::CastResult, shape::Shape};

pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub material: MaterialShared,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32, material: MaterialShared) -> Self {
        Self { position, radius,  material }
    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: crate::math::Ray) -> Option<CastResult> {
        let oc = ray.origin() - self.position;
        let a = Vec3::dot(ray.direction(), ray.direction());
        let b = 2.0 * Vec3::dot(oc, ray.direction());
        let c = Vec3::dot(oc, oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        } else {
            let distance_traversed = (-b - discriminant.sqrt()) / (2.0 * a);
            let intersection_point = ray.point_at_parameter(distance_traversed);

            if distance_traversed < 0.0 {
                return Some(CastResult::MISS);
            }

            let normal = (intersection_point - self.position).normalized();

            return Some(CastResult {
                distance_traversed,
                intersection_point,
                normal,
                material: self.material.clone(),
            });
        }
    }

    fn material(&self) -> &crate::scene::material::Material {
        return self.material.get();
    }
}
