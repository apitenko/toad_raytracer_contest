use std::{intrinsics::size_of, marker::PhantomData, mem::MaybeUninit};

use const_guards::guard;
use rand::Rng;

use crate::{
    constants::FLOAT_ERROR,
    math::{
        cone::Cone,
        sphere::{sphere_bbox_intersection, Sphere},
        Ray, Vec3,
    },
    primitives::{
        bounding_box::{BoundingBox, BBOX_PAD},
        cast_result::{CastIntersectionResult, CastResult, ConeCastResult, ConeCastResultStep},
        plane::Plane,
        shape::Shape,
        triangle::Triangle,
    },
    util::fixed_array::FixedArray,
};

use super::{acceleration_structure::AccelerationStructure, svogi::NodeGIInfo};

pub struct OctreeNode {
    pub triangles: Vec<Triangle>,
    pub children: [*mut OctreeNode; 8],
    pub bbox: BoundingBox,
    pub bbox_padded: BoundingBox,
    // pub gi_info: NodeGIInfo, // todo: mutex/atomic
}

impl AccelerationStructure for Octree {
    fn push_triangle(&mut self, insert_triangle: Triangle) {
        self.push_triangle_(insert_triangle);
    }

    fn single_cast(&self, ray: Ray, inside: bool) -> CastIntersectionResult {
        let result = Self::recursive_intersection(&ray, self.root, &ray, Self::ROOT_DEPTH);
        if let Some(result) = result {
            return result;
        }
        else {
            return CastIntersectionResult::MISS;
        }
    }

    fn cone_cast(&self, cone: Cone) -> ConeCastResult {
        self.cone_cast_(cone)
    }

    fn inject_emittance_data(&mut self, ray: Ray) {
        self.inject_emittance_data(ray);
    }
}

impl OctreeNode {
    // pub fn empty() -> Self {
    //     Self {
    //         triangles: Vec::new(),
    //         children: [std::ptr::null_mut(); 8],
    //         bbox: BoundingBox {
    //             center: Vec3::ZERO,
    //             mid_planes: [
    //                 Plane::new(Vec3::X_AXIS, 0.0),
    //                 Plane::new(Vec3::Y_AXIS, 0.0),
    //                 Plane::new(Vec3::Z_AXIS, 0.0),
    //             ],
    //             min: Vec3::ZERO,
    //             max: Vec3::ZERO,
    //         },
    //         bbox_padded: BoundingBox {
    //             center: Vec3::ZERO,
    //             mid_planes: [
    //                 Plane::new(Vec3::X_AXIS, 0.0),
    //                 Plane::new(Vec3::Y_AXIS, 0.0),
    //                 Plane::new(Vec3::Z_AXIS, 0.0),
    //             ],
    //             min: Vec3::ZERO,
    //             max: Vec3::ZERO,
    //         },
    //         gi_info: NodeGIInfo {
    //             directional_density: [0.0, 0.0, 0.0],
    //             directional_emittance: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
    //         },
    //     }
    // }

    #[guard(CHILD_INDEX <= 7 && CHILD_INDEX >= 0)]
    fn make_child_bbox<const CHILD_INDEX: i32>(bbox: BoundingBox) -> Self {
        let len = bbox.max - bbox.min;
        let len_h = len / 2.0;
        let len_h_x = Vec3::from_f32([len_h.x(), 0.0, 0.0, 0.0]);
        let len_h_y = Vec3::from_f32([0.0, len_h.y(), 0.0, 0.0]);
        let len_h_z = Vec3::from_f32([0.0, 0.0, len_h.z(), 0.0]);
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
        let bbox = BoundingBox::new(bbox_min_max.0, bbox_min_max.1);
        let bbox_padded = bbox.padded(BBOX_PAD);
        Self {
            triangles: Vec::with_capacity(0),
            children: [std::ptr::null_mut(); 8],
            bbox,
            bbox_padded,
            // gi_info: NodeGIInfo {
            //     directional_density: [0.0, 0.0, 0.0],
            //     directional_emittance: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
            // },
        }
    }
}

pub struct Octree {
    memory: FixedArray<OctreeNode, { Self::NODES_MAX }>, // node memory
    root: *mut OctreeNode,                               // tree root node
}

lazy_static::lazy_static! {
    static ref ROOT_RESOLUTION: f32 = {
        f32::powi(2.0, Octree::ROOT_DEPTH)
    };

    static ref ROOT_BBOX: BoundingBox = BoundingBox::new(
        Vec3::new([
            -*ROOT_RESOLUTION / 2.0,
            -*ROOT_RESOLUTION / 2.0,
            -*ROOT_RESOLUTION / 2.0,
        ]),
        Vec3::new([
            *ROOT_RESOLUTION / 2.0,
            *ROOT_RESOLUTION / 2.0,
            *ROOT_RESOLUTION / 2.0,
        ]),
    );
}

impl Octree {
    const NODES_MAX: usize = 5 * 1024 * 1024;
    // unused; rust-analyzer will show the size of the memory allocated
    const _TOTAL_OCTREE_SIZE_BYTES: usize = Self::NODES_MAX * size_of::<OctreeNode>();


    const ROOT_DEPTH: i32 = 12;
    const MIN_DEPTH: i32 = -6;

    pub fn empty() -> Self {
        let mut memory = FixedArray::<OctreeNode, { Self::NODES_MAX }>::with_capacity();

        let root = memory.push(OctreeNode {
            triangles: Vec::new(),
            children: [std::ptr::null_mut(); 8],
            bbox: ROOT_BBOX.clone(),
            bbox_padded: ROOT_BBOX.clone().padded(BBOX_PAD),
            // gi_info: NodeGIInfo {
            //     directional_density: [0.0, 0.0, 0.0],
            //     directional_emittance: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
            // },
        });

        Self { memory, root }
    }

    pub fn push_triangle_(&mut self, insert_triangle: Triangle) {
        // traverse the tree: if intersected, continue traversing (or add if the current level reached)
        let current_node = self.root;

        let result = self.tree_traversal_insert(current_node, &insert_triangle, Self::ROOT_DEPTH);

        if let Err(e) = result {
            panic!("tree_traversal_insert failed");
        }
    }

    fn tree_traversal_insert(
        &mut self,
        current_node: *mut OctreeNode,
        insert_triangle: &Triangle,
        current_level: i32,
    ) -> anyhow::Result<()> {
        debug_assert!(!current_node.is_null());
        unsafe {
            if current_level <= Self::MIN_DEPTH {
                if BoundingBox::intersects_triangle(&(*current_node).bbox, insert_triangle) {
                    // insert to the current node
                    (*current_node).triangles.push(insert_triangle.clone());
                    return Ok(());
                }
            } else {
                if (*current_node).children[0].is_null() {
                    self.subdivide(current_node)?;
                }
                // intersect with each, go recursive if true
                for child in (*current_node).children {
                    if BoundingBox::intersects_triangle(&(*child).bbox, insert_triangle) {
                        self.tree_traversal_insert(child, &insert_triangle, current_level - 1)?;
                    }
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

    fn intersect_triangles(node: *mut OctreeNode, ray: &Ray) -> CastIntersectionResult {
        debug_assert!(!node.is_null());
        unsafe {
            let inside = false;

            // intersect with all triangles, return nearest CastResult or None
            let cast_result = (*node)
                .triangles
                .iter()
                .filter_map(|item| unsafe {
                    //
                    (item).intersect(*ray, inside)
                })
                .fold(CastIntersectionResult::MISS, |acc, item| {
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
    }

    fn recursive_intersection(
        original_ray: &Ray,
        node: *mut OctreeNode,
        ray: &Ray,
        current_level: i32
    ) -> Option<CastIntersectionResult> {
        unsafe {
            debug_assert!(!node.is_null());

            if current_level <= Self::MIN_DEPTH {
                return Some(Self::intersect_triangles(node, original_ray));
            }
            // let current_cast_result = 
            // if !current_cast_result.has_missed() {
            //     return current_cast_result;
            // }

            // keep intersecting children by nearest, until any intersection is found
            // return nearest of children_cast_result and current_cast_result

            let planes = &(*node).bbox.mid_planes;
            let mut origin = ray.origin();
            let direction = ray.direction();
            let bbox_padded = &(*node).bbox_padded;
            // let center =  (*node).bbox.

            struct _Sides {
                pub X: bool,
                pub Y: bool,
                pub Z: bool,
            }

            let mut side = _Sides {
                X: origin.x() - planes[0].distance >= 0.0,
                Y: origin.y() - planes[1].distance >= 0.0,
                Z: origin.z() - planes[2].distance >= 0.0,
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

            debug_assert!(xDist >= 0.0);
            debug_assert!(yDist >= 0.0);
            debug_assert!(zDist >= 0.0);

            fn side_to_index(side: &_Sides) -> usize {
                (if side.Z { 4 } else { 0 })
                    | (if side.Y { 2 } else { 0 })
                    | (if side.X { 1 } else { 0 })
            }

            for _ in 0..4 {
                let idx = side_to_index(&side);

                {
                    let child_node = (*node).children[idx];
                    if !child_node.is_null() {
                        let ret = Self::recursive_intersection(
                            original_ray,
                            child_node,
                            &Ray::new(origin, direction, f32::INFINITY),
                            current_level - 1,
                        );

                        if let Some(ret) = ret {
                            // intersection found, return as it's always the nearest

                            // current_cast_result = minimum_of_two_cast_results(current_cast_result, ret);
                            return Some(ret);
                            // assert!(current_cast_result.has_missed());
                            // return ret;
                        }
                    }
                }

                let minDist = f32::min(f32::min(xDist, yDist), zDist);
                if f32::is_infinite(minDist) {
                    return None;
                }

                debug_assert!(minDist >= 0.0);
                origin = ray.origin() + minDist * direction;

                if !bbox_padded.contains(origin) {
                    return None;
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
            return None;
        }
    }

    fn cone_cast_(&self, cone: Cone) -> ConeCastResult {
        todo!();
        // let mut accumulated_color = Vec3::ZERO;
        // let mut remaining_density: f32 = 1.0;
        // for sphere in cone.iter() {
        //     let step = Self::recursive_sphere_intersection(&sphere, &cone.origin, self.root);
        //     accumulated_color += step.accumulated_color * remaining_density;
        //     remaining_density -= step.accumulated_density;
        //     remaining_density = f32::clamp(remaining_density, 0.0, 1.0);
        //     if remaining_density <= 0.0 {
        //         break;
        //     }
        // }

        // return ConeCastResult { accumulated_color };
    }

    #[inline]
    fn recursive_sphere_intersection(
        sphere: &Sphere,
        origin: &Vec3,
        node: *mut OctreeNode,
    ) -> ConeCastResultStep {
        todo!();
        // todo: simplify/approximate
        // unsafe {
        //     if !sphere_bbox_intersection(&sphere, &(*node).bbox) {
        //         return ConeCastResultStep::empty();
        //     }

        //     let bbox = &(*node).bbox;

        //     fn projection_area_approx(
        //         origin: &Vec3,
        //         point0: Vec3,
        //         point1: Vec3,
        //         point2: Vec3,
        //     ) -> f32 {
        //         let cosy = Vec3::dot(*origin - point0, *origin - point1);
        //         let cosz = Vec3::dot(*origin - point0, *origin - point2);
        //         return cosy * cosz;
        //     };
        //     // todo: question my life choices
        //     let projection_area = {
        //         let projection_area_x = {
        //             let point0 = Vec3::from_f32([bbox.center.x(), bbox.min.y(), bbox.min.z(), 0.0]);
        //             let point1 = Vec3::from_f32([bbox.center.x(), bbox.min.y(), bbox.max.z(), 0.0]);
        //             let point2 = Vec3::from_f32([bbox.center.x(), bbox.max.y(), bbox.min.z(), 0.0]);
        //             projection_area_approx(origin, point0, point1, point2)
        //         };
        //         let projection_area_y = {
        //             let point0 = Vec3::from_f32([bbox.min.x(), bbox.center.y(), bbox.min.z(), 0.0]);
        //             let point1 = Vec3::from_f32([bbox.min.x(), bbox.center.y(), bbox.max.z(), 0.0]);
        //             let point2 = Vec3::from_f32([bbox.max.x(), bbox.center.y(), bbox.min.z(), 0.0]);
        //             projection_area_approx(origin, point0, point1, point2)
        //         };
        //         let projection_area_z = {
        //             let point0 = Vec3::from_f32([bbox.min.x(), bbox.min.y(), bbox.center.z(), 0.0]);
        //             let point1 = Vec3::from_f32([bbox.max.x(), bbox.min.y(), bbox.center.z(), 0.0]);
        //             let point2 = Vec3::from_f32([bbox.min.x(), bbox.max.y(), bbox.center.z(), 0.0]);
        //             projection_area_approx(origin, point0, point1, point2)
        //         };

        //         [projection_area_x, projection_area_y, projection_area_z]
        //     };

        //     let current_densities = [
        //         (*node).gi_info.directional_density[0] * projection_area[0],
        //         (*node).gi_info.directional_density[1] * projection_area[1],
        //         (*node).gi_info.directional_density[2] * projection_area[2],
        //     ];

        //     let current_colors = [
        //         (*node).gi_info.directional_emittance[0] * projection_area[0],
        //         (*node).gi_info.directional_emittance[1] * projection_area[1],
        //         (*node).gi_info.directional_emittance[2] * projection_area[2],
        //     ];

        //     let current_color = current_colors[0] + current_colors[1] + current_colors[2];
        //     let current_density =
        //         current_densities[0] + current_densities[1] + current_densities[2];

        //     let mut result = ConeCastResultStep {
        //         accumulated_color: current_color,
        //         accumulated_density: current_density,
        //     };

        //     if !(*node).children[0].is_null() {
        //         for child in (*node).children {
        //             result = ConeCastResultStep::merge(
        //                 result,
        //                 Self::recursive_sphere_intersection(sphere, origin, child),
        //             );
        //         }
        //     }

        //     return result;
        // }
    }
}
