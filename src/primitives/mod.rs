use crate::math::Vec3;



pub struct Camera {
    position: Vec3,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }
}