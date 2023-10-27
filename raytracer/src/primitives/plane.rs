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

        if f32::abs(denom) > 0.0001
        // your favorite epsilon
        {
            let center = self.normal * self.distance;
            let t: f32 = Vec3::dot(center - origin, self.normal) / denom;
            if t < 0.0 {
                return f32::INFINITY;
            }
            return t;
        }
        return f32::INFINITY;
    }

    pub const fn new(normal: Vec3, distance: f32) -> Self {
        Plane { distance, normal }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Vec3;

    use super::Plane;

    #[test]
    fn Plane_RayDistance() {
        assert_eq!(
            {
                let plane = Plane::new(Vec3::X_AXIS, 100.0);
                let distance = plane.RayDistance(Vec3::new([0.0, 0.0, 0.0]), -Vec3::X_AXIS);
                distance.is_infinite()
            },
            true
        );
        assert_eq!(
            {
                let plane = Plane::new(Vec3::X_AXIS, 100.0);
                let distance = plane.RayDistance(Vec3::new([0.0, 0.0, 0.0]), Vec3::X_AXIS);
                distance > 99.9 && distance < 100.1
            },
            true
        );
        assert_eq!(
            {
                let plane = Plane::new(Vec3::X_AXIS, -100.0);
                let distance = plane.RayDistance(Vec3::new([0.0, 0.0, 0.0]), Vec3::new([-2.0, 0.0, -2.0]).normalized());
                distance > 141.41 && distance < 141.47
            },
            true
        );
    }
}
