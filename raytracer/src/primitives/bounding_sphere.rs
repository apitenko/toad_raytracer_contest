// use crate::{math::{Ray, Vec3}, scene::material::MaterialShared};

// use super::{shape::Shape, cast_result::CastResult};

// pub struct BoundingSphere {
//     pub position: Vec3,
//     pub radius: f32,
// }

// // Same as Sphere, but adds intersections inside the sphere
// impl Shape for BoundingSphere {
//     fn intersect(&self, ray: Ray, inside: bool) -> Option<CastResult> {
//         todo!();
//         let oc = ray.origin() - self.position;
//         let a = Vec3::dot(ray.direction(), ray.direction());
//         let b = 2.0 * Vec3::dot(oc, ray.direction());
//         let c = Vec3::dot(oc, oc) - self.radius * self.radius;
//         let discriminant = b * b - 4.0 * a * c;

//         if discriminant < 0.0 {
//             return None;
//         } else {
//             let distance_traversed_t0 = (-b - discriminant.sqrt()) / (2.0 * a);
//             let distance_traversed_t1 = (-b + discriminant.sqrt()) / (2.0 * a);

//             if distance_traversed_t0 < 0.0 && distance_traversed_t1 < 0.0 {
//                 return Some(CastResult::MISS);
//             }

//             // smaller and positive t
//             // outside and inside intersections
//             let distance_traversed = if distance_traversed_t0 > 0.0 && distance_traversed_t1 > 0.0 {
//                 // f32::min(distance_traversed_t0, distance_traversed_t1)
//             } else if distance_traversed_t0 < 0.0 && distance_traversed_t1 > 0.0 {
//                 // distance_traversed_t1
//             } else if distance_traversed_t0 > 0.0 && distance_traversed_t1 < 0.0 {
//                 // distance_traversed_t0
//             } else {
//                 return None;
//             };

//             // let intersection_point = ray.point_at_parameter(distance_traversed);

//             return Some(CastResult {
//                 distance_traversed: f32::MAX,
//                 intersection_point: Vec3::ZERO,
//                 normal: Vec3::UP,
//                 material: MaterialShared::null(),
//                 uv: (0.0, 0.0),
//             });
//         }
//     }
// }
