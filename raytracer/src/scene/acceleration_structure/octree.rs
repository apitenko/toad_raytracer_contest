use std::{intrinsics::size_of, marker::PhantomData, mem::MaybeUninit};

use rand::Rng;

use crate::{
    math::{random::random_in_unit_sphere, Ray, Vec3},
    primitives::{
        bounding_box::BoundingBox, cast_result::CastResult, plane::Plane, shape::Shape,
        triangle::Triangle,
    },
    util::unresizable_array::UnresizableArray,
};

use super::acceleration_structure::AccelerationStructure;

pub struct OctreeNode {
    pub triangles: Vec<Triangle>,
    pub children: [*mut OctreeNode; 8],
    pub bbox: BoundingBox,
}

impl AccelerationStructure for Octree {
    fn push_triangle(&mut self, insert_triangle: Triangle) {
        self.push_triangle_(insert_triangle);
    }
    fn single_cast(&self, ray: Ray, inside: bool) -> CastResult {
        // println!("----------------");
        return Self::recursive_intersection(self.root, ray);
    }
}

impl OctreeNode {
    pub fn empty() -> Self {
        Self {
            triangles: Vec::new(),
            children: [std::ptr::null_mut(); 8],
            bbox: BoundingBox {
                center: Vec3::ZERO,
                mid_planes: [
                    Plane::new(Vec3::X_AXIS, 0.0),
                    Plane::new(Vec3::Y_AXIS, 0.0),
                    Plane::new(Vec3::Z_AXIS, 0.0),
                ],
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            },
        }
    }

    pub fn make_child_bbox<const CHILD_INDEX: i32>(bbox: BoundingBox) -> Self {
        let len = bbox.max - bbox.min;
        let len_h = len / 2.0;
        let len_h_x = Vec3::new([len_h.x(), 0.0, 0.0]);
        let len_h_y = Vec3::new([0.0, len_h.y(), 0.0]);
        let len_h_z = Vec3::new([0.0, 0.0, len_h.z()]);
        let middle = bbox.min + len_h;

        let bbox_min_max = match CHILD_INDEX {
            0 => (bbox.min, middle),
            1 => (bbox.min + len_h_x, middle + len_h_x),
            2 => (bbox.min + len_h_y, middle + len_h_y),
            3 => (bbox.min + len_h_x + len_h_y, middle + len_h_x + len_h_y),
            4 => (bbox.min + len_h_z, middle + len_h_z),
            5 => (bbox.min + len_h_x + len_h_z, middle + len_h_x + len_h_z),
            6 => (bbox.min + len_h_y + len_h_z, middle + len_h_y + len_h_z),
            7 => (
                bbox.min + len_h_x + len_h_y + len_h_z,
                middle + len_h_x + len_h_y + len_h_z,
            ),
            _ => panic!("wrong index"),
        };
        Self {
            triangles: Vec::with_capacity(0),
            children: [std::ptr::null_mut(); 8],
            bbox: BoundingBox::new(bbox_min_max.0, bbox_min_max.1),
        }
    }
}

pub struct Octree {
    // meshes: UnresizableArray<Mesh, { Self::MESHES_MAX }>, // mesh memory
    memory: UnresizableArray<OctreeNode, { Self::NODES_MAX }>, // node memory
    root: *mut OctreeNode,                                     // tree root node
}

lazy_static::lazy_static! {
    static ref ROOT_RESOLUTION: f32 = {
        f32::powi(2.0, Octree::ROOT_DEPTH)
    };

    static ref ROOT_BBOX: BoundingBox = BoundingBox::new(
        Vec3::new([
            -*ROOT_RESOLUTION,
            -*ROOT_RESOLUTION,
            -*ROOT_RESOLUTION,
        ]),
        Vec3::new([
            *ROOT_RESOLUTION,
            *ROOT_RESOLUTION,
            *ROOT_RESOLUTION,
        ]),
    );
}

impl Octree {
    const MESHES_MAX: usize = 4000;
    const NODES_MAX: usize = 3200000;

    // unused; rust-analyzer will show the size of the memory allocated
    const NODES_MEMORY_ALLOCATED_BYTES: usize = Self::NODES_MAX * size_of::<OctreeNode>();

    const ROOT_DEPTH: i32 = 12;
    const MIN_DEPTH: i32 = -30;

    pub fn empty() -> Self {
        let mut memory = UnresizableArray::<OctreeNode, { Self::NODES_MAX }>::with_capacity();

        let root = memory.push(OctreeNode {
            triangles: Vec::new(),
            children: [std::ptr::null_mut(); 8],
            bbox: ROOT_BBOX.clone(),
        });

        Self { memory, root }
    }

    pub fn push_triangle_(&mut self, insert_triangle: Triangle) {
        let insert_bbox = BoundingBox::from_triangle(&insert_triangle);

        // find size
        let max_side = {
            let delta = insert_bbox.max - insert_bbox.min;
            f32::max(f32::max(delta.x(), delta.y()), delta.z())
        };
        // min level is size-1
        let level = f32::log2(max_side).ceil() as i32 - 1;

        // traverse the tree: if intersected, continue traversing (or add if the current level reached)
        let current_node = self.root;

        let result = self.tree_traversal_insert(
            current_node,
            Self::ROOT_DEPTH,
            &insert_triangle,
            &insert_bbox,
            level,
        );

        if let Err(e) = result {
            panic!("tree_traversal_insert failed");
        }
    }

    fn tree_traversal_insert(
        &mut self,
        current_node: *mut OctreeNode,
        current_level: i32,
        insert_triangle: &Triangle,
        insert_bbox: &BoundingBox,
        insert_level: i32,
    ) -> anyhow::Result<()> {
        debug_assert!(!current_node.is_null());
        unsafe {
            if current_level <= insert_level {
                // insert to the current node
                (*current_node).triangles.push(insert_triangle.clone());
                return Ok(());
            }

            if current_level <= Self::MIN_DEPTH {
                // insert to the current node
                (*current_node).triangles.push(insert_triangle.clone());
                return Ok(());
            }

            // find 8 or split by 8
            if (*current_node).children[0].is_null() {
                self.subdivide(current_node)?;
            }
            // intersect with each, go recursive if true
            for child in (*current_node).children {
                if BoundingBox::intersects(insert_bbox, &(*child).bbox) {
                    self.tree_traversal_insert(
                        child,
                        current_level - 1,
                        &insert_triangle,
                        &insert_bbox,
                        insert_level,
                    )?;
                }
            }
            Ok(())
        }
    }

    fn subdivide(&mut self, node: *mut OctreeNode) -> anyhow::Result<()> {
        unsafe {
            if (*node).children[0].is_null() {
                (*node).children = [
                    self.memory
                        .push(OctreeNode::make_child_bbox::<0>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<1>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<2>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<3>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<4>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<5>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<6>((*node).bbox)),
                    self.memory
                        .push(OctreeNode::make_child_bbox::<7>((*node).bbox)),
                ];
            }
        }
        Ok(())
    }

    fn intersect_triangles(node: *mut OctreeNode, ray: Ray) -> CastResult {
        debug_assert!(!node.is_null());
        unsafe {
            let inside = false;

            // println!("-- intersection -- {:?} \n -- {:?} ", (*node).bbox, ray);
            // if ! ((*node).triangles.is_empty()) {
            //     println!("FSDFSDF");
            // }

            // intersect with all triangles, return nearest CastResult or None
            let cast_result = (*node)
                .triangles
                .iter()
                .filter_map(|item| unsafe {
                    //
                    (item).intersect(ray, inside)
                })
                .fold(CastResult::MISS, |acc, item| {
                    if acc.distance_traversed > item.distance_traversed
                        && item.distance_traversed > 0.001
                    // && item.distance_traversed <= ray.max_distance()
                    {
                        return item;
                    } else {
                        return acc;
                    }
                });

            return cast_result;
        }
    }

    fn recursive_intersection(node: *mut OctreeNode, ray: Ray) -> CastResult {
        unsafe {
            if node.is_null() {
                return CastResult::MISS;
            }
            let current_cast_result = Self::intersect_triangles(node, ray);
            // return current_cast_result;
            // if !current_cast_result.has_missed() {
            //     println!("HIT");
            // }

            // keep intersecting children by nearest, until any intersection is found
            // return nearest of children_cast_result and current_cast_result

            let planes = &(*node).bbox.mid_planes;
            let mut origin = ray.origin();
            let direction = ray.direction();
            let bbox = &(*node).bbox;
            // let center =  (*node).bbox.

            struct _Sides {
                pub X: bool,
                pub Y: bool,
                pub Z: bool,
            }

            let mut side = {
                _Sides {
                    X: Vec3::dot(origin, planes[0].normal) - planes[0].distance >= 0.0,
                    Y: Vec3::dot(origin, planes[1].normal) - planes[1].distance >= 0.0,
                    Z: Vec3::dot(origin, planes[2].normal) - planes[2].distance >= 0.0,
                }
            };

            let mut xDist: f32 = if side.X == (direction.x() < 0.0) {
                planes[0].RayDistance(origin, direction)
            } else {
                f32::INFINITY
            };
            let mut yDist: f32 = if side.Y == (direction.y() < 0.0) {
                planes[1].RayDistance(origin, direction)
            } else {
                f32::INFINITY
            };
            let mut zDist: f32 = if side.Z == (direction.z() < 0.0) {
                planes[2].RayDistance(origin, direction)
            } else {
                f32::INFINITY
            };

            fn side_to_index(side: &_Sides) -> usize {
                (if side.Z { 4 } else { 0 }
                    | if side.Y { 2 } else { 0 }
                    | if side.X { 1 } else { 0 })
            }

            fn minimum_of_two_cast_results(left: CastResult, right: CastResult) -> CastResult {
                if left.distance_traversed > right.distance_traversed {
                    return right;
                } else {
                    return left;
                }
            }

            for _ in 0..4 {
                let idx = side_to_index(&side);

                let ret = Self::recursive_intersection(
                    (*node).children[idx],
                    Ray::new(origin, direction, f32::INFINITY),
                );
                if !ret.has_missed() {
                    return minimum_of_two_cast_results(current_cast_result, ret);
                }

                let minDist = f32::min(f32::min(xDist, yDist), zDist);
                if f32::is_infinite(minDist) {
                    return current_cast_result;
                }

                origin = ray.origin() + direction * minDist;

                if !bbox.contains(origin) {
                    return current_cast_result;
                }

                if minDist == xDist {
                    side.X = !side.X;
                    xDist = f32::INFINITY;
                } else if minDist == yDist {
                    side.Y = !side.Y;
                    yDist = f32::INFINITY;
                } else if minDist == zDist {
                    side.Z = !side.Z;
                    zDist = f32::INFINITY;
                }
            }
            return current_cast_result;
        }
    }
}
