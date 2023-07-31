use crate::math::Vec3;

use super::{cast_result::CastResult, shape::Shape};

pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub color: Vec3,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32, color: Vec3) -> Self {
        Self { position, radius, color }
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

            // TODO: Sample the color
            // let color = {
            //     let N = (intersection_point - Vec3::BACK).normalized();
            //     0.5 * (N + Vec3::ONE)
            // };
            let color = self.color;

            let normal = (self.position - intersection_point).normalized();

            return Some(CastResult {
                distance_traversed,
                intersection_point,
                normal,
                color,
            });
        }
    }
}
