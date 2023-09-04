use crate::{math::Vec3, scene::material::MaterialShared};

use super::{
    bounding_box::BoundingBox, cast_result::CastResult, shape::Shape, sphere::Sphere,
    triangle::Triangle,
};

pub struct Mesh {
    pub material: MaterialShared,
    pub triangles: Vec<Triangle>,
    pub aabb: BoundingBox,
    pub bounding_sphere: Sphere,
}

// impl Mesh {
//     pub fn new(triangles: Vec<Triangle>, material: MaterialShared, bounding_sphere: Sphere) -> Self {
//         Self {
//             material,
//             triangles,
//             bounding_sphere
//         }
//     }
// }

impl Shape for Mesh {
    fn intersect(&self, ray: crate::math::Ray, inside: bool) -> Option<CastResult> {
        // let bounding_volume_cast = self.bounding_sphere.intersect(ray, false);
        // if let None = bounding_volume_cast {
        //     return None;
        // }

        let cast_result = self
            .triangles
            .iter()
            .filter_map(|item| unsafe { (*item).intersect(ray, inside) })
            .fold(CastResult::MISS, |acc, item| {
                if acc.distance_traversed > item.distance_traversed
                    && item.distance_traversed > 0.001
                    && item.distance_traversed <= ray.max_distance()
                {
                    return item;
                } else {
                    return acc;
                }
            });

        if cast_result.has_missed() {
            return None;
        } else {
            return Some(CastResult {
                material: self.material.clone(),
                ..cast_result
            });
        }
    }
}
