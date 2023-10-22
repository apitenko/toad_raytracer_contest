use crate::{primitives::{triangle::Triangle, cast_result::{CastResult, ConeCastResult}}, math::{cone::Cone, Vec3}};

use super::acceleration_structure::AccelerationStructure;
use crate::primitives::shape::Shape;

pub struct FlatArray {
    triangles: Vec<Triangle>
}

impl FlatArray {
    pub fn empty() -> Self {
        Self {
            triangles: Vec::with_capacity(4096)
        }
    }
}

impl AccelerationStructure for FlatArray {
    fn push_triangle(&mut self, insert_triangle: Triangle) {
        self.triangles.push(insert_triangle);
    }

    fn single_cast(&self, ray: crate::math::Ray, inside: bool) -> CastResult {
        
        let cast_result = self
        .triangles
        .iter()
        .filter_map(|item| unsafe {
            //
            (*item).intersect(ray, inside)
        })
        .fold(CastResult::MISS, |acc, item| {
            if (acc.distance_traversed > item.distance_traversed)
                & (item.distance_traversed > 0.001)
                & (item.distance_traversed <= ray.max_distance())
            {
                return item;
            } else {
                return acc;
            }
        });

        return cast_result;
    }

    fn cone_cast(&self, cone: Cone) -> ConeCastResult {
        // unimplementable
        ConeCastResult {
            accumulated_color: Vec3::ZERO,
        }
    }

    fn inject_emittance_data(&mut self, ray: crate::math::Ray) {
        // unimplementable
    }
}