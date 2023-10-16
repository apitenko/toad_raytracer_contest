use crate::constants::MONTE_CARLO_THRESHOLD_BOUNCES;

use super::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, max_distance: f32) -> Self {
        Self {
            direction: direction.normalized(),
            origin,
            max_distance,
        }
    }

    pub fn origin(&self) -> Vec3 {
        return self.origin;
    }

    pub fn direction(&self) -> Vec3 {
        return self.direction;
    }

    pub fn max_distance(&self) -> f32 {
        return self.max_distance;
    }

    pub fn point_at_parameter(&self, t: f32) -> Vec3 {
        return Vec3::add(self.origin, Vec3::multiply_by_f32(self.direction, t));
    }
}

pub fn reflect(vector: Vec3, normal: Vec3) -> Vec3 {
    return (vector - 2.0 * Vec3::dot(vector, normal) * normal).normalized();
}

pub fn refract(incident: Vec3, surface_normal: Vec3, refractiveness_ratio: f32) -> Vec3 {
    let n = surface_normal;
    let i = incident;
    let eta = refractiveness_ratio;

    let dot = Vec3::dot(n, i);
    let dot_squared = dot * dot;
    let k = 1.0 - eta * eta * (1.0 - dot_squared);
    if k < 0.0 {
        return Vec3::ZERO;
    } else {
        return eta * i - (eta * dot + k.sqrt()) * n;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RayRefractionState {
    /// Ray is currently inside a solid material.
    InsideMaterial {
        current_outside_fresnel_coefficient: f32,
    },
    /// Ray is outside, going through air.
    TraversingAir,
}

pub struct RayBounce {
    pub ray: Ray,
    pub current_bounces: i32,
    pub distance: f32,
    pub refraction_state: RayRefractionState,
}

impl RayBounce {
    pub fn default_from_ray(ray: Ray) -> Self {
        Self {
            ray,
            current_bounces: 0,
            distance: 1.0,
            // remaining_depth: MAX_DEPTH,
            refraction_state: RayRefractionState::InsideMaterial {
                current_outside_fresnel_coefficient: 9.9,
            },
        }
    }

    #[inline]
    pub fn monte_carlo_reached(&self) -> bool {
        self.current_bounces > MONTE_CARLO_THRESHOLD_BOUNCES
    }
}

impl Into<RayBounce> for Ray {
    fn into(self) -> RayBounce {
        RayBounce::default_from_ray(self)
    }
}
