use crate::math::{Mat44, Ray, Vec3};

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

    pub fn from_matrices(view_matrix: Mat44, projection_matrix: Mat44) -> Self {
        let view_matrix = view_matrix.inverse();
        let projection_matrix = projection_matrix.inverse();
        // inverse projection (from 01 to World Space)
        // then rotate & translate
        let mvp = view_matrix * projection_matrix;

        let bottom_left_p0 = mvp.transform_point(Vec3::from_f32([-1.0, -1.0, 0.0, 1.0]));
        let bottom_left_p1 = mvp.transform_point(Vec3::from_f32([-1.0, -1.0, 1.0, 1.0]));
        let bottom_right_p0 = mvp.transform_point(Vec3::from_f32([1.0, -1.0, 0.0, 1.0]));
        let bottom_right_p1 = mvp.transform_point(Vec3::from_f32([1.0, -1.0, 1.0, 1.0]));
        let top_left_p0 = mvp.transform_point(Vec3::from_f32([-1.0, 1.0, 0.0, 1.0]));
        let top_left_p1 = mvp.transform_point(Vec3::from_f32([-1.0, 1.0, 1.0, 1.0]));

        let bottom_left = (bottom_left_p1 - bottom_left_p0).normalized();
        let bottom_right = (bottom_right_p1 - bottom_right_p0).normalized();
        let top_left = (top_left_p1 - top_left_p0).normalized();

        let origin = view_matrix * Vec3::from_f32([0.0, 0.0, 0.0, 1.0]);

        Camera {
            lower_left_corner: bottom_left,
            height_in_units: top_left - bottom_left,
            width_in_units: bottom_right - bottom_left,
            origin,
        }
    }

    pub fn ray(&self, u: f32, v: f32) -> Ray {
        let ray = Ray::new(
            self.origin,
            self.lower_left_corner + u * self.width_in_units + v * self.height_in_units,
            f32::MAX,
        );
        return ray;
    }
}
