use crate::math::Vec3;

pub fn color(ray: &Vec3) -> Vec3 {
    let ray_normalized = ray.normalized();
    let t = 0.5 * (ray_normalized.y() + 1.0);
    return (1.0 - t) * Vec3::ONE + t * Vec3::COLOR_CALL_PARAMETERS;
}