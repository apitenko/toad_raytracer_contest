use rand::Rng;

use crate::{
    math::{Ray, Vec3, random::random_in_unit_sphere},
    primitives::{cast_result::CastResult, shape::Shape, sphere::Sphere}, constants::MAX_BOUNCES,
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

    pub fn traverse(&self, ray: Ray) -> SVOIterator {
        return SVOIterator {
            current_ray: ray,
            root: self as *const SVORoot,
            remaining_bounces: MAX_BOUNCES
        };
    }

    pub fn single_cast(&self, ray: Ray) -> CastResult {
        // TODO: Scene traversal logic w/ SVOIterator

        let cast_result = self
            .root
            .shapes
            .iter()
            .filter_map(|item| unsafe { (**item).intersect(ray) })
            .fold(CastResult::MISS, |acc, item| {
                if acc.distance_traversed > item.distance_traversed {
                    return item;
                } else {
                    return acc;
                }
            });

        return cast_result;
    }
}

pub struct SVOIterator {
    remaining_bounces: u32,
    current_ray: Ray,
    root: *const SVORoot,
}

impl Iterator for SVOIterator {
    type Item = CastResult;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bounces <= 0 {
            return None;
        } else {
            let cast_result = unsafe { (*self.root).single_cast(self.current_ray) };

            self.remaining_bounces -= 1;
            if cast_result.distance_traversed == f32::INFINITY {
                self.remaining_bounces = 0;
            }

            let rnd = random_in_unit_sphere();

            self.current_ray = Ray::new(cast_result.intersection_point + rnd, cast_result.normal + rnd);

            return Some(cast_result);
        }
    }
}
