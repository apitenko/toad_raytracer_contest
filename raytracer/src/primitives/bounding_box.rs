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
            fn Project(points: &[Vec3], axis: Vec3) -> (f32, f32) {
                // let axis = axis;
                let mut min = f32::INFINITY;
                let mut max = f32::NEG_INFINITY;
                for p in points {
                    let val = Vec3::dot(axis, *p);
                    if val < min {
                        min = val
                    }
                    if val > max {
                        max = val
                    }
                }
                return (min, max);
            }

            let aabb_vertices = aabb.as_8_points();

            let triangle_normal = triangle.calculate_geometry_normal();

            // Test the box normals (x-, y- and z-axes)
            const BOX_NORMALS: [Vec3;3] = [Vec3::X_AXIS, Vec3::Y_AXIS, Vec3::Z_AXIS];

            for i in 0..3 {
                let (triangle_min, triangle_max) = Project(&triangle.vertices, BOX_NORMALS[i]);
                if triangle_max < aabb.min.get()[i] || triangle_min > aabb.max.get()[i] {
                    return false; // No intersection possible.
                }
            }

            // Test the triangle normal
            let triangle_offset = Vec3::dot(triangle_normal, triangle.vertices[0]);
            let (box_min, box_max) = Project(&aabb_vertices, triangle_normal);
            if box_max < triangle_offset || box_min > triangle_offset {
                return false; // No intersection possible.
            }
            // Test the nine edge cross-products
            let triangle_edges = [
                triangle.vertices[0] - triangle.vertices[1],
                triangle.vertices[1] - triangle.vertices[2],
                triangle.vertices[2] - triangle.vertices[0],
            ];
            for i in 0..3 {
                for j in 0..3 {
                    // The box normals are the same as it's edge tangents
                    // ? no need to normalize since we're comparing points projected onto the same non-normalized axis
                    let axis = Vec3::cross(triangle_edges[i], BOX_NORMALS[j]);//.normalized();
                    let (box_min, box_max) = Project(&aabb_vertices, axis);
                    let (triangle_min, triangle_max) = Project(&triangle.vertices, axis);
                    if box_max < triangle_min || box_min > triangle_max {
                        return false; // No intersection possible
                    }
                }
            }
            // No separating axis found.
            return true;
        }
    }

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
