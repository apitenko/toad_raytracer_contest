use std::f32::consts::{PI, TAU};

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
                uv: self.uv(intersection_point)
            });
        }
    }

    fn material(&self) -> &crate::scene::material::Material {
        return self.material.get();
    }

    fn uv(&self, intersection_point: Vec3) -> (f32, f32) {
        let point = (intersection_point - self.position) / self.radius;
        let [dx, dy, dz, ..] = point.extract();
        let u = 0.5 + f32::atan2(dz, dx) / TAU;
        let v = 0.5 - f32::asin(dy) / PI;
        return (u,v);
    }
}


/*

        let point = intersection_point - self.position;
        let [dx, dy, dz, ..] = point.extract();
        
        let theta = f32::atan2(dx, dz);
        let radius = intersection_point.length();
        let phi = f32::acos(dy / radius);
      
        let raw_u = theta / (2.0 * PI);
      
        let u = 1.0 - (raw_u + 0.5);
      
        let v = 1.0 - phi / PI;
      
        return (u,v);
*/