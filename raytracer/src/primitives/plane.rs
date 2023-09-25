use crate::math::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    // Gives the distance along the ray (+direction) where intersection with the plane occurs
    pub fn RayDistance(&self, origin: Vec3, direction: Vec3) -> f32 {
        let denom = Vec3::dot(self.normal, direction);

        if f32::abs(denom) > 0.0001 // your favorite epsilon
        {
            let center = self.normal * self.distance;
            let t: f32 = Vec3::dot(center - origin, self.normal) / denom;
            return t;
        }
        return f32::INFINITY;
    }

    pub const fn new(normal: Vec3, distance: f32) -> Self {
        Plane { distance, normal }
    }
}