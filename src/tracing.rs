use crate::{
    math::{Ray, Vec3},
    primitives::sphere::Sphere,
};

pub fn color(ray: &Ray) -> Vec3 {
    let sphere = Sphere::new(Vec3::new([0.0, 0.0, -1.0]), 0.5);
    let t = hit_sphere(&sphere, &ray);

    if t > 0.0 {
        let N = (ray.point_at_parameter(t) - Vec3::BACK).normalized();
        return 0.5 * (N + Vec3::ONE);
    }

    // "skybox"
    let ray_normalized = ray.direction().normalized();
    let t = 0.5 * (ray_normalized.y() + 1.0);
    return (1.0 - t) * Vec3::ONE + t * Vec3::COLOR_CALL_PARAMETERS;
}

pub fn hit_sphere(sphere: &Sphere, ray: &Ray) -> f32 {
    let oc = ray.origin() - sphere.position;
    let a = Vec3::dot(ray.direction(), ray.direction());
    let b = 2.0 * Vec3::dot(oc, ray.direction());
    let c = Vec3::dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return -1.0;
    } else {
        return (-b - discriminant.sqrt()) / (2.0 * a);
    }
}
