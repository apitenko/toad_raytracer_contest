use crate::{math::Vec3, scene::material::MaterialShared};

use super::{plane::Plane, triangle::Triangle};

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

    // todo: vectorize
    pub fn from_triangle(tri: &Triangle) -> Self {
        let mut min_x = f32::min(
            f32::min(tri.vertices[0].x(), tri.vertices[1].x()),
            tri.vertices[2].x(),
        );
        let mut min_y = f32::min(
            f32::min(tri.vertices[0].y(), tri.vertices[1].y()),
            tri.vertices[2].y(),
        );
        let mut min_z = f32::min(
            f32::min(tri.vertices[0].z(), tri.vertices[1].z()),
            tri.vertices[2].z(),
        );

        let mut max_x = f32::max(
            f32::max(tri.vertices[0].x(), tri.vertices[1].x()),
            tri.vertices[2].x(),
        );
        let mut max_y = f32::max(
            f32::max(tri.vertices[0].y(), tri.vertices[1].y()),
            tri.vertices[2].y(),
        );
        let mut max_z = f32::max(
            f32::max(tri.vertices[0].z(), tri.vertices[1].z()),
            tri.vertices[2].z(),
        );

        const EPSILON: f32 = 0.001;
        if max_x - min_x < EPSILON {
            max_x += EPSILON;
            min_x -= EPSILON;
        }
        if max_y - min_y < EPSILON {
            max_y += EPSILON;
            min_y -= EPSILON;
        }
        if max_z - min_z < EPSILON {
            max_z += EPSILON;
            min_z -= EPSILON;
        }

        const TRI_PADDING_EPSILON: f32 = 0.001;
        const PADDING: Vec3 = Vec3::new([TRI_PADDING_EPSILON, TRI_PADDING_EPSILON, TRI_PADDING_EPSILON]);

        let min = Vec3::from_f32([min_x, min_y, min_z, 0.0]) - PADDING;
        let max = Vec3::from_f32([max_x, max_y, max_z, 0.0]) + PADDING;

        Self::new(min, max)
    }

    #[must_use]
    pub fn intersects(a: &Self, b: &Self) -> bool {
        (a.min.x() <= b.max.x() && a.max.x() >= b.min.x()) && // .
        (a.min.y() <= b.max.y() && a.max.y() >= b.min.y()) && // .
        (a.min.z() <= b.max.z() && a.max.z() >= b.min.z()) // .
    }
    #[must_use]
    pub fn intersects_padded(a: &Self, b: &Self, padding: Vec3) -> bool {
        (a.min.x() - padding.x() <= b.max.x() && a.max.x() + padding.x() >= b.min.x()) && // .
        (a.min.y() - padding.y() <= b.max.y() && a.max.y() + padding.y() >= b.min.y()) && // .
        (a.min.z() - padding.z() <= b.max.z() && a.max.z() + padding.z() >= b.min.z())
        // .
    }

    pub fn from_gltf(aabb: gltf::mesh::BoundingBox) -> Self {
        Self::new(Vec3::new(aabb.min), Vec3::new(aabb.max))
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
