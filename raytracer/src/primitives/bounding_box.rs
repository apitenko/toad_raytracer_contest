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
        return bbox.padded(0.003);
    }

    #[must_use]
    pub fn intersects(a: &Self, b: &Self) -> bool {
        ((a.min.x() <= b.max.x()) & (a.max.x() >= b.min.x())) & // .
        ((a.min.y() <= b.max.y()) & (a.max.y() >= b.min.y())) & // .
        ((a.min.z() <= b.max.z()) & (a.max.z() >= b.min.z())) // .
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
