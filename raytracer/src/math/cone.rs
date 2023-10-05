use super::{vec3::Vec3, sphere::Sphere};

#[derive(Clone, Copy, Debug)]
pub struct Cone {
    pub origin: Vec3,
    pub direction: Vec3,
    pub tan_angle_2: f32,
}

pub struct ConeIterator {
    pub distance: f32,
    pub current_steps: usize,
    pub origin: Vec3,
    pub direction: Vec3,
    pub tan_angle_2: f32,
}

const CONE_MAX_STEPS: usize = 16;
const CONE_INITIAL_DISTANCE: f32 = 0.05;

impl Iterator for ConeIterator {
    type Item = Sphere;
    fn next(&mut self) -> Option<Sphere> {
        if self.current_steps > CONE_MAX_STEPS {
            return None;
        }
        self.current_steps += 1;

        let radius = self.distance / self.tan_angle_2;
        let position = self.direction * self.distance;
        self.distance += radius;
        return Some(Sphere { position, radius });
    }
}

impl Cone {
    pub fn new(origin: Vec3, direction: Vec3, angle: f32) -> Self {
        Self {
            origin,
            direction,
            tan_angle_2: f32::tan(angle) / 2.0,
        }
    }

    pub fn iter(&self) -> ConeIterator {
        ConeIterator {
            distance: CONE_INITIAL_DISTANCE,
            current_steps: 0,
            origin: self.origin,
            direction: self.direction,
            tan_angle_2: self.tan_angle_2,
        }
    }
}
