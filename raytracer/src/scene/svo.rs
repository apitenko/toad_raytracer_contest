use crate::{
    math::{Ray, Vec3},
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

    pub fn traverse(&self, ray: Ray) -> SVOIterator {
        return SVOIterator {
            current_ray: ray,
            root: self as *const SVORoot,
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
    current_ray: Ray,
    root: *const SVORoot,
}

impl Iterator for SVOIterator {
    type Item = CastResult;
    fn next(&mut self) -> Option<Self::Item> {

        // TODO: actually iterate
        let cast_result = unsafe { (*self.root).single_cast(self.current_ray) };
        return Some(cast_result);
    }
}
