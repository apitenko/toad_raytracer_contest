use std::{marker::PhantomData, mem::MaybeUninit};

use rand::Rng;

use crate::{
    math::{random::random_in_unit_sphere, Ray, Vec3},
    primitives::{
        cast_result::CastResult, mesh::Mesh, shape::Shape, sphere::Sphere, triangle::Triangle,
    },
    util::unresizable_array::UnresizableArray,
};

pub struct SVONode {
    // shapes: Vec<*const dyn Shape>,
    meshes: Vec<*const Mesh>,
    // TODO: actual tree
}

impl SVONode {
    pub fn empty() -> Self {
        Self { meshes: Vec::new() }
    }
}

pub struct SVORoot {
    meshes: UnresizableArray<Mesh, { Self::MESHES_MAX }>, // mesh memory
    memory: UnresizableArray<SVONode, { Self::NODES_MAX }>, // node memory
    root: SVONode,                                        // tree root node
}

impl SVORoot {
    const MESHES_MAX: usize = 2000;
    const NODES_MAX: usize = 2000;

    pub fn empty() -> Self {
        Self {
            memory: UnresizableArray::<SVONode, { Self::NODES_MAX }>::with_capacity(),
            root: SVONode::empty(),
            meshes: UnresizableArray::<Mesh, { Self::MESHES_MAX }>::with_capacity(),
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        // todo: octree search & insert
        let mesh_ptr = self.meshes.push(mesh);
        self.root.meshes.push(mesh_ptr);
    }

    // pub fn push_triangle(&mut self, triangle: Triangle) {
    //     self.root.shapes.push(triangle);
    // }

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
            .meshes
            .iter()
            .filter_map(|item| unsafe {
                //
                (**item).intersect(ray, inside)
            })
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
