use std::sync::Arc;

use crate::{
    math::{random::random_in_unit_sphere, Ray, Vec3},
    primitives::cast_result::CastResult,
};

pub struct RefractiveGlass {
    albedo: Vec3,
    reflectivity: f32,
}

impl RefractiveGlass {
    pub fn new(albedo: Vec3, reflectivity: f32) -> Self {
        Self {
            albedo,
            reflectivity,
        }
    }
}

impl Material for RefractiveGlass {
    fn scatter(&self, ray_in: &Ray, cast_result: &CastResult) -> MaterialScatterResult {
        let reflected_normal = reflect(ray_in.direction().normalized(), cast_result.normal);
        let scattered_ray = Ray::new(cast_result.intersection_point, reflected_normal, f32::MAX);
        let attenuation = self.albedo;
        let is_valid = Vec3::dot(scattered_ray.direction(), cast_result.normal) > 0.0;

        return MaterialScatterResult {
            reflected: scattered_ray,
            refracted: None,
            is_valid: true,
            attenuation,
        };
    }
}

// TODO: Sample the color
// let color = {
//     let N = (intersection_point - Vec3::BACK).normalized();
//     0.5 * (N + Vec3::ONE)
// };

pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub const fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, cast_result: &CastResult) -> MaterialScatterResult {
        let rnd = random_in_unit_sphere().normalized();
        let scattered = Ray::new(
            cast_result.intersection_point,
            cast_result.normal + rnd,
            f32::MAX,
        );
        let attenuation = self.albedo;
        return MaterialScatterResult {
            reflected: scattered,
            refracted: None,
            is_valid: true,
            attenuation,
        };
    }
}

pub struct Metal {
    albedo: Vec3,
}

impl Metal {
    pub const fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, cast_result: &CastResult) -> MaterialScatterResult {
        let reflected_normal = reflect(ray_in.direction().normalized(), cast_result.normal);
        let scattered_ray = Ray::new(cast_result.intersection_point, reflected_normal, f32::MAX);
        let attenuation = self.albedo;
        let is_valid = Vec3::dot(scattered_ray.direction(), cast_result.normal) > 0.0;

        return MaterialScatterResult {
            reflected: scattered_ray,
            refracted: None,
            is_valid: true,
            attenuation,
        };
    }
}

pub struct MaterialScatterResult {
    pub reflected: Ray,
    pub refracted: Option<Ray>,
    pub attenuation: Vec3,
    pub is_valid: bool,
}

pub trait Material {
    fn scatter(&self, ray_in: &Ray, cast_result: &CastResult) -> MaterialScatterResult;
}

fn reflect(vector: Vec3, normal: Vec3) -> Vec3 {
    return vector - 2.0 * Vec3::dot(vector, normal) * normal;
}

#[derive(Clone)]
pub struct MaterialShared {
    mat: *const dyn Material,
}

unsafe impl Send for MaterialShared {}
unsafe impl Sync for MaterialShared {}

impl MaterialShared {
    pub fn new(mat: *const dyn Material) -> Self {
        Self { mat }
    }

    pub fn get(&self) -> &dyn Material {
        unsafe {
            return &*self.mat as &dyn Material;
        }
    }

    const DEFAULT_MAT_IMPL: Lambertian = Lambertian::new(Vec3::ONE);
    pub const DEFAULT_MAT: Self = Self {
        mat: &Self::DEFAULT_MAT_IMPL,
    };
}
