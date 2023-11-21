use crate::{math::Vec3, scene::material::MaterialShared};

use super::{plane::Plane, triangle::Triangle};
use std::{arch::x86_64::*, fmt::Debug, mem::MaybeUninit};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub center: Vec3,
    pub mid_planes: [Plane; 3],
    pub min: Vec3,
    pub max: Vec3,
}

pub const BBOX_PAD: f32 = 0.0000;

impl BoundingBox {
    // True if the point lies within the bounding box
    pub fn contains(&self, point: Vec3) -> bool {
        return (self.min.x() <= point.x())
            & (self.max.x() >= point.x())
            & (self.min.y() <= point.y())
            & (self.max.y() >= point.y())
            & (self.min.z() <= point.z())
            & (self.max.z() >= point.z());
    }

    pub const fn new(min: Vec3, max: Vec3) -> Self {
        // Dear Lord, please forgive for I am gonna constexpr
        let center = Vec3::add_constexpr(
            min,
            Vec3::divide_by_f32_constexpr(Vec3::subtract_constexpr(max, min), 2.0),
        );

        let mid_planes = [
            Plane::new(Vec3::X_AXIS, center.x()),
            Plane::new(Vec3::Y_AXIS, center.y()),
            Plane::new(Vec3::Z_AXIS, center.z()),
        ];
        Self {
            center,
            mid_planes,
            min,
            max,
        }
    }

    pub fn from_triangle(tri: &Triangle) -> Self {
        let max = Vec3::max(Vec3::max(tri.vertices[0], tri.vertices[1]), tri.vertices[2]);
        let min = Vec3::min(Vec3::min(tri.vertices[0], tri.vertices[1]), tri.vertices[2]);

        let bbox = Self::new(min, max);
        return bbox.padded(BBOX_PAD);
    }

    #[must_use]
    pub fn intersects(a: &Self, b: &Self) -> bool {
        ((a.min.x() <= b.max.x()) & (a.max.x() >= b.min.x())) & // .
        ((a.min.y() <= b.max.y()) & (a.max.y() >= b.min.y())) & // .
        ((a.min.z() <= b.max.z()) & (a.max.z() >= b.min.z())) // .
    }

    #[must_use]
    pub fn intersects_triangle(aabb: &Self, triangle: &Triangle) -> bool {
        {
            #[inline]
            fn AABB_CenterExtents(aabb: &BoundingBox) -> (Vec3, Vec3) {                
                let extents = aabb.center - aabb.min;
                (aabb.center, extents)
            }
            
            #[inline]
            fn Test(points: &[Vec3], aabb_extents: Vec3, axis: Vec3) -> bool {

                let p0 = Vec3::dot(points[0], axis);
                let p1 = Vec3::dot(points[1], axis);
                let p2 = Vec3::dot(points[2], axis);
                            
                let r = aabb_extents.x() * f32::abs(Vec3::dot(Vec3::X_AXIS, axis)) +
                aabb_extents.y() * f32::abs(Vec3::dot(Vec3::Y_AXIS, axis)) +
                aabb_extents.z() * f32::abs(Vec3::dot(Vec3::Z_AXIS, axis));

                let p = Vec3::new([p0, p1, p2]);
                
                if f32::max(-p.max_component_3(), p.min_component_3()) > r {
                    // This means BOTH of the points of the projected triangle
                    // are outside the projected half-length of the AABB
                    // Therefore the axis is seperating and we can exit
                    return false;
                }
                return true;
            }

            let triangle_edges = [
                triangle.vertices[0] - triangle.vertices[1],
                triangle.vertices[1] - triangle.vertices[2],
                triangle.vertices[2] - triangle.vertices[0],
            ];
            let (aabb_center, aabb_extents) = AABB_CenterExtents(aabb);
            let triangle_vertices = [
                triangle.vertices[0] - aabb_center,
                triangle.vertices[1] - aabb_center,
                triangle.vertices[2] - aabb_center,
            ];

            // ! Test 9 axes

            for i in 0..3 {
                for j in 0..3 {
                    // The box normals are the same as it's edge tangents
                    let axis = Vec3::cross(triangle_edges[i], BOX_NORMALS[j]);
                    if !Test(&triangle_vertices, aabb_extents, axis) {
                        return false;
                    }
                }
            }

            // ! Test 3 AABB axes

            // Test the box normals (x-, y- and z-axes)
            const BOX_NORMALS: [Vec3;3] = [Vec3::X_AXIS, Vec3::Y_AXIS, Vec3::Z_AXIS];

            for i in 0..3 {
                if !Test(&triangle_vertices, aabb_extents, BOX_NORMALS[i]) {
                    return false;
                }
            }

            // ! Test 1 triangle normal axis

            let triangle_normal = Vec3::cross(triangle_edges[0],triangle_edges[1]);
            if !Test(&triangle_vertices, aabb_extents, triangle_normal) {
                return false;
            }

            // No separating axis found.
            return true;
        }
    }

    #[deprecated]
    pub fn as_8_points(&self) -> [Vec3; 8] {
        [
            self.min,
            Vec3::new([self.max.x(), self.min.y(), self.min.z()]),
            Vec3::new([self.min.x(), self.max.y(), self.min.z()]),
            Vec3::new([self.max.x(), self.max.y(), self.min.z()]),
            Vec3::new([self.min.x(), self.min.y(), self.max.z()]),
            Vec3::new([self.max.x(), self.min.y(), self.max.z()]),
            Vec3::new([self.min.x(), self.max.y(), self.max.z()]),
            self.max,
        ]
    }

    pub fn from_gltf(aabb: gltf::mesh::BoundingBox) -> Self {
        Self::new(Vec3::new(aabb.min), Vec3::new(aabb.max))
    }

    #[inline]
    pub fn padded(&self, pad: f32) -> Self {
        let pad = Vec3::from_f32([pad, pad, pad, 0.0]);
        Self::new(self.min - pad, self.max + pad)
    }
    // pub fn bounding_sphere(&self) -> Sphere {
    //     let min = self.min;
    //     let max = self.max;
    //     let position = min + (max - min) / 2.0;
    //     let radius = (max - position).length();
    //     Sphere {
    //         position,
    //         radius,
    //         material: MaterialShared::null(),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use crate::{math::Vec3, primitives::bounding_box::BoundingBox};

    #[test]
    fn bbox_intersect() {
        assert_eq!(
            {
                let a = BoundingBox::new(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([1.0, 1.0, 1.0]));
                let b = BoundingBox::new(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([2.0, 2.0, 2.0]));
                BoundingBox::intersects(&a, &b)
            },
            true
        );

        assert_eq!(
            {
                let a = BoundingBox::new(Vec3::new([-1.0, -1.0, -1.0]), Vec3::new([6.0, 6.0, 6.0]));
                let b = BoundingBox::new(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([2.0, 2.0, 2.0]));
                BoundingBox::intersects(&a, &b)
            },
            true
        );

        assert_eq!(
            {
                let a = BoundingBox::new(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([1.0, 1.0, 1.0]));
                let b = BoundingBox::new(Vec3::new([0.4, 0.4, 0.4]), Vec3::new([0.6, 0.6, 0.6]));
                BoundingBox::intersects(&a, &b)
            },
            true
        );

        assert_eq!(
            {
                let a = BoundingBox::new(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([1.0, 1.0, 1.0]));
                let b = BoundingBox::new(Vec3::new([1.4, 1.4, 1.4]), Vec3::new([1.6, 1.6, 1.6]));
                BoundingBox::intersects(&a, &b)
            },
            false
        );

        assert_eq!(
            {
                let a = BoundingBox::new(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([1.0, 1.0, 1.0]));
                let b =
                    BoundingBox::new(Vec3::new([-1.4, -1.4, 1.4]), Vec3::new([-1.6, -1.6, 1.6]));
                BoundingBox::intersects(&a, &b)
            },
            false
        );
    }
}
