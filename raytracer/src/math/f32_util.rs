pub trait Saturatable {
    fn saturate(&self) -> f32;
}

impl Saturatable for f32 {
    fn saturate(&self) -> f32 {
        self.clamp(f32::EPSILON, 1.0 - f32::EPSILON)
    }
}

pub trait FloatWrapTo01 {
    fn wrap_01(self) -> Self;
}

impl FloatWrapTo01 for f32 {
    fn wrap_01(self) -> Self {
        self - Self::floor(self)
    }
}