use rand::Rng;

use crate::{
    math::{random::random_in_unit_sphere, Ray, Vec3},
    primitives::{cast_result::CastResult, shape::Shape, sphere::Sphere},
};

pub struct SVONode {
    shapes: Vec<*const dyn Shape>,
    // TODO: actual tree
}

pub struct SVORoot {
    root: SVONode,
}

impl SVORoot {
    pub fn empty() -> Self {
        Self {
            root: SVONode { shapes: Vec::new() },
        }
    }

    pub fn push_shape(&mut self, shape: *const dyn Shape) {
        self.root.shapes.push(shape);
    }

    // pub fn traverse(&self, ray: Ray) -> SVOIterator {
    //     return SVOIterator {
    //         current_ray: ray,
    //         root: self as *const SVORoot,
    //         remaining_bounces: MAX_BOUNCES,
    //         reflectivity: 1.0,
    //     };
    // }

    pub fn single_cast(&self, ray: Ray, inside: bool) -> CastResult {
        // TODO: Scene traversal logic w/ SVOIterator

        let cast_result = self
            .root
            .shapes
            .iter()
            .filter_map(|item| unsafe { (**item).intersect(ray, inside) })
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

        return cast_result;
    }
}
