use std::{marker::PhantomData, mem::MaybeUninit};

use rand::Rng;

use crate::{
    math::{random::random_in_unit_sphere, Ray, Vec3},
    primitives::{
        cast_result::CastResult, mesh::Mesh, shape::Shape, sphere::Sphere, triangle::Triangle,
    },
};

pub struct UnresizableArray<T, const TCapacity: usize>
where
    T: Sized,
{
    data: Box<[T; TCapacity]>,
    current_index: usize,
}

impl<T, const TCapacity: usize> UnresizableArray<T, TCapacity>
where
    T: Sized,
{
    const MAX_SIZE: usize = TCapacity;
    pub fn with_capacity() -> Self
    where
        [(); Self::MAX_SIZE]:,
    {
        unsafe {
            Self {
                data: Box::new_uninit().assume_init(),
                current_index: 0,
            }
        }
    }

    pub fn push(&mut self, mut item: T) -> *const T {
        if self.current_index >= Self::MAX_SIZE {
            panic!("UnresizableArray is out of memory");
        }

        std::mem::swap(&mut self.data[self.current_index], &mut item);
        std::mem::forget(item);

        let ptr = &self.data[self.current_index] as *const T;
        self.current_index += 1;
        return ptr;
    }
}

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
    meshes: UnresizableArray<Mesh, 2000>,    // mesh memory
    memory: UnresizableArray<SVONode, 2000>, // node memory
    root: SVONode,                           // tree root node
}

impl SVORoot {
    pub fn empty() -> Self {
        Self {
            memory: UnresizableArray::<SVONode, 2000>::with_capacity(),
            root: SVONode::empty(),
            meshes: UnresizableArray::<Mesh, 2000>::with_capacity(),
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
