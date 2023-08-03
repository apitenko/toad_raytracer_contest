use crate::{
    math::Vec3,
    scene::material::{Material, MaterialShared},
};

use super::{cast_result::CastResult, shape::Shape};

pub struct Quad {
    // position: Vec3,
    translated_geometry: [Vec3; 4],
    material: MaterialShared,
}

const UV_SCALE: f32 = 0.2;
const PLANE_SIZE: f32 = 50.0;

impl Quad {
    pub fn new(position: Vec3, geometry: [Vec3; 4], material: MaterialShared) -> Self {
        Self {
            translated_geometry: [
                (geometry[0] + position),
                (geometry[1] + position),
                (geometry[2] + position),
                (geometry[3] + position),
            ],
            material,
        }
    }

    pub const DEFAULT_GEOMETRY: [Vec3; 4] = [
        // ccw
        Vec3::new([-PLANE_SIZE, 0.0, -PLANE_SIZE]),
        Vec3::new([-PLANE_SIZE, 0.0, PLANE_SIZE]),
        Vec3::new([PLANE_SIZE, 0.0, PLANE_SIZE]),
        Vec3::new([PLANE_SIZE, 0.0, -PLANE_SIZE]),
    ];
}

impl Shape for Quad {
    fn intersect(&self, ray: crate::math::Ray) -> Option<CastResult> {
        let geom = &self.translated_geometry;

        // 1.
        let delta_s10: Vec3 = geom[1] - geom[0];
        let delta_s20: Vec3 = geom[2] - geom[0];
        let surface_normal: Vec3 = Vec3::cross(delta_s10, delta_s20);

        // 2.
        let ray_direction: Vec3 = ray.direction();

        let surface_normal_dot_ray_direction: f32 = Vec3::dot(surface_normal, ray_direction);

        if f32::abs(surface_normal_dot_ray_direction) < 0.001 {
            return None;
        }
        
        let t: f32 = Vec3::dot(-surface_normal, ray.origin() - geom[0]) / surface_normal_dot_ray_direction;
        if t <= 0.0 {
            return None;
        }
        let intersection_point: Vec3 = ray.origin() + (ray_direction * t);

        // print!("d {:?}", intersection_point);
        // 3.
        let dMS1: Vec3 = intersection_point - (geom[0]);
        let u = Vec3::dot(dMS1, delta_s10);
        let v = Vec3::dot(dMS1, delta_s20);

        // 4.
        let is_intersection_happened = (u >= 0.0 && u <= Vec3::dot(delta_s10, delta_s10)
             && v >= 0.0 && v <= Vec3::dot(delta_s20, delta_s20));

        if !is_intersection_happened {
            return None;
        }

        let intersection_point = intersection_point;
        let distance_traversed = (ray_direction * t).length();
        // print!("d {:?}", intersection_point);

        return Some(CastResult {
            distance_traversed,
            intersection_point,
            uv: (u, v),
            normal: Vec3::UP,
            material: self.material.clone(),
        });
    }

    fn material(&self) -> &Material {
        return self.material.get();
    }

    fn uv(&self, intersection_point: Vec3) -> (f32, f32) {
        // pee pee poo poo
        panic!("quad.uv");
        (0.0, 0.0)
    }
}
