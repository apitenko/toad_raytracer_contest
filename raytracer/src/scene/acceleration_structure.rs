use std::{marker::PhantomData, mem::MaybeUninit};

use rand::Rng;

use crate::{
    math::{random::random_in_unit_sphere, Ray, Vec3},
    primitives::{
        bounding_box::BoundingBox, cast_result::CastResult, mesh::Mesh, shape::Shape,
        sphere::Sphere, triangle::Triangle,
    },
    util::unresizable_array::UnresizableArray,
};

pub struct SVONode {
    pub triangles: Vec<Triangle>,
    pub children: [*mut SVONode; 8],
    pub bbox: BoundingBox,
}

impl SVONode {
    pub fn empty() -> Self {
        Self {
            triangles: Vec::new(),
            children: [std::ptr::null_mut(); 8],
            bbox: BoundingBox {
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            },
        }
    }

    pub fn empty_bbox_split_by_index(bbox: BoundingBox, index: i32) -> Self {
        let len = bbox.max - bbox.min;
        let len_h = len / 2.0;
        let len_h_x = Vec3::new([len_h.x(), 0.0, 0.0]);
        let len_h_y = Vec3::new([0.0, len_h.y(), 0.0]);
        let len_h_z = Vec3::new([0.0, 0.0, len_h.z()]);
        let middle = bbox.min + len_h;

        let bbox_new = match index {
            0 => BoundingBox {
                min: bbox.min,
                max: middle,
            },
            1 => BoundingBox {
                min: bbox.min + len_h_x,
                max: middle + len_h_x,
            },
            2 => BoundingBox {
                min: bbox.min + len_h_y,
                max: middle + len_h_y,
            },
            3 => BoundingBox {
                min: bbox.min + len_h_x + len_h_y,
                max: middle + len_h_x + len_h_y,
            },
            4 => BoundingBox {
                min: bbox.min + len_h_z,
                max: middle + len_h_z,
            },
            5 => BoundingBox {
                min: bbox.min + len_h_x + len_h_z,
                max: middle + len_h_x + len_h_z,
            },
            6 => BoundingBox {
                min: bbox.min + len_h_y + len_h_z,
                max: middle + len_h_y + len_h_z,
            },
            7 => BoundingBox {
                min: bbox.min + len_h_x + len_h_y + len_h_z,
                max: middle + len_h_x + len_h_y + len_h_z,
            },
            _ => panic!("empty_bbox_split_by_index"),
        };
        Self {
            triangles: Vec::with_capacity(0),
            children: [std::ptr::null_mut(); 8],
            bbox: bbox_new
        }
    }
}

pub struct SVORoot {
    // meshes: UnresizableArray<Mesh, { Self::MESHES_MAX }>, // mesh memory
    memory: UnresizableArray<SVONode, { Self::NODES_MAX }>, // node memory
    root: SVONode,                                          // tree root node
}

impl SVORoot {
    const MESHES_MAX: usize = 4000;
    const NODES_MAX: usize = 4000;

    const ROOT_DEPTH: i32 = 12;
    const MIN_DEPTH: i32 = -4;
    const ROOT_RESOLUTION: f32 = f32::powi(2.0, Self::ROOT_DEPTH);

    const ROOT_BBOX: BoundingBox = BoundingBox {
        min: Vec3::new([
            -Self::ROOT_RESOLUTION,
            -Self::ROOT_RESOLUTION,
            -Self::ROOT_RESOLUTION,
        ]),
        max: Vec3::new([
            Self::ROOT_RESOLUTION,
            Self::ROOT_RESOLUTION,
            Self::ROOT_RESOLUTION,
        ]),
    };

    pub fn empty() -> Self {
        Self {
            memory: UnresizableArray::<SVONode, { Self::NODES_MAX }>::with_capacity(),
            root: SVONode {
                triangles: Vec::new(),
                children: [std::ptr::null_mut(); 8],
                bbox: Self::ROOT_BBOX,
            },
            // meshes: UnresizableArray::<Mesh, { Self::MESHES_MAX }>::with_capacity(),
        }
    }

    pub fn push_triangle(&mut self, insert_triangle: Triangle) {
        let insert_bbox = BoundingBox::from_triangle(&insert_triangle);

        // find size
        let max_side = {
            let delta = insert_bbox.max - insert_bbox.min;
            f32::max(f32::max(delta.x(), delta.y()), delta.z())
        };
        // min level is size-1
        let level = f32::log2(max_side).ceil() as i32 - 1;

        // traverse the tree: if intersected, continue traversing (or add if the current level reached)
        let current_node = &mut self.root as *mut _;

        let result = self.tree_traversal_insert(
            current_node,
            Self::ROOT_DEPTH,
            insert_triangle,
            insert_bbox,
            level,
        );

        if let Err(e) = result {
            panic!("tree_traversal_insert failed");
        }
    }

    fn tree_traversal_insert(
        &mut self,
        current_node: *mut SVONode,
        current_level: i32,
        insert_triangle: Triangle,
        insert_bbox: BoundingBox,
        insert_level: i32,
    ) -> anyhow::Result<()> {
        debug_assert!(!current_node.is_null());
        unsafe {
            if insert_level <= current_level {
                // insert to the current node
                (*current_node).triangles.push(insert_triangle);
                return Ok(());
            }

            if current_level <= Self::MIN_DEPTH {
                // insert to the current node
                (*current_node).triangles.push(insert_triangle);
                return Ok(());
            }

            // find 8 or split by 8
            if (*current_node).children[0].is_null() {
                self.subdivide(current_node)?;
            }
            // intersect with each, go recursive if true
            for child in (*current_node).children {
                if BoundingBox::intersects(&(*current_node).bbox, &(*child).bbox) {
                    self.tree_traversal_insert(
                        child,
                        current_level - 1,
                        insert_triangle,
                        insert_bbox,
                        insert_level,
                    )?;
                }
            }
            Ok(())
        }
    }

    fn subdivide(&mut self, node: *mut SVONode) -> anyhow::Result<()> {
        unsafe {
            if (*node).children[0].is_null() {
                (*node).children = [
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 0)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 1)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 2)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 3)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 4)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 5)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 6)),
                    self.memory
                        .push(SVONode::empty_bbox_split_by_index((*node).bbox, 7)),
                ];
            }
        }
        Ok(())
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
