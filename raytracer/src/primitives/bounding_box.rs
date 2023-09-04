use crate::{math::Vec3, scene::material::MaterialShared};

use super::{sphere::Sphere, triangle::Triangle};

pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn from_gltf(aabb: gltf::mesh::BoundingBox) -> Self {
        Self {
            min: Vec3::new(aabb.min),
            max: Vec3::new(aabb.max),
        }
    }
    pub fn bounding_sphere(&self) -> Sphere {
        let min = self.min;
        let max = self.max;
        let position = min + (max - min) / 2.0;
        let radius = (max - position).length();
        Sphere {
            position,
            radius,
            material: MaterialShared::null(),
        }
    }

    // todo: vectorize
    pub fn from_triangle(tri: &Triangle) -> Self {
        let min = {
            let min_x = f32::min(
                f32::min(tri.vertices[0].x(), tri.vertices[1].x()),
                tri.vertices[2].x(),
            );
            let min_y = f32::min(
                f32::min(tri.vertices[0].y(), tri.vertices[1].y()),
                tri.vertices[2].y(),
            );
            let min_z = f32::min(
                f32::min(tri.vertices[0].z(), tri.vertices[1].z()),
                tri.vertices[2].z(),
            );

            Vec3::from_f32([min_x, min_y, min_z, 0.0])
        };
        let max = {
            let max_x = f32::max(
                f32::max(tri.vertices[0].x(), tri.vertices[1].x()),
                tri.vertices[2].x(),
            );
            let max_y = f32::max(
                f32::max(tri.vertices[0].y(), tri.vertices[1].y()),
                tri.vertices[2].y(),
            );
            let max_z = f32::max(
                f32::max(tri.vertices[0].z(), tri.vertices[1].z()),
                tri.vertices[2].z(),
            );

            Vec3::from_f32([max_x, max_y, max_z, 0.0])
        };

        Self { min, max }
    }
}
