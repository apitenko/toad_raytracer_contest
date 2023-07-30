use crate::math::{Ray, Vec3};

pub struct Camera {
    pub lower_left_corner: Vec3,
    pub width_in_units: Vec3,
    pub height_in_units: Vec3,
    pub origin: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            lower_left_corner: Vec3::new([-2.0, -1.0, -1.0]),
            width_in_units: Vec3::new([4.0, 0.0, 0.0]),
            height_in_units: Vec3::new([0.0, 2.0, 0.0]),
            origin: Vec3::ZERO,
        }
    }

    pub fn ray(&self, u: f32, v: f32) -> Ray {
        let ray = Ray::new(
            self.origin,
            self.lower_left_corner + u * self.width_in_units + v * self.height_in_units,
        );
        return ray;
    }
}
