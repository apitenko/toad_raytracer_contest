pub mod random;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    data: [f32; 3],
}

impl Vec3 {
    pub const ZERO: Self = Self {
        data: [0.0, 0.0, 0.0],
    };

    pub const ONE: Self = Self {
        data: [1.0, 1.0, 1.0],
    };

    // pub const UP: Self = Self {
    //     data: [0.0, 1.0, 0.0],
    // };

    pub const BACK: Self = Self {
        data: [0.0, 0.0, -1.0],
    };

    #[inline]
    pub const fn new(data: [f32; 3]) -> Self {
        Self { data }
    }

    #[inline]
    pub fn x(&self) -> f32 {
        return self.data[0];
    }

    #[inline]
    pub fn y(&self) -> f32 {
        return self.data[1];
    }

    #[inline]
    pub fn z(&self) -> f32 {
        return self.data[2];
    }

    #[inline]
    pub fn add(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[0] + right.data[0],
            left.data[1] + right.data[1],
            left.data[2] + right.data[2],
        ]);
    }

    #[inline]
    pub fn subtract(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[0] - right.data[0],
            left.data[1] - right.data[1],
            left.data[2] - right.data[2],
        ]);
    }

    #[inline]
    pub fn multiply_components(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[0] * right.data[0],
            left.data[1] * right.data[1],
            left.data[2] * right.data[2],
        ]);
    }

    #[inline]
    pub fn multiply_by_f32(left: Vec3, right: f32) -> Self {
        return Self::new([
            left.data[0] * right,
            left.data[1] * right,
            left.data[2] * right,
        ]);
    }

    #[inline]
    pub fn divide_by_f32(left: Vec3, right: f32) -> Self {
        return Self::new([
            left.data[0] / right,
            left.data[1] / right,
            left.data[2] / right,
        ]);
    }

    #[inline]
    pub fn dot(left: Vec3, right: Vec3) -> f32 {
        return (left.data[0] * right.data[0])
            + (left.data[1] * right.data[1])
            + (left.data[2] * right.data[2]);
    }

    #[inline]
    pub fn cross(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[1] * right.data[2] - left.data[2] * right.data[1],
            -(left.data[0] * right.data[2] - left.data[2] * right.data[0]),
            left.data[0] * right.data[1] - left.data[1] * right.data[0],
        ]);
    }

    #[inline]
    pub fn length(&self) -> f32 {
        return self.squared_length().sqrt();
    }

    #[inline]
    pub fn squared_length(&self) -> f32 {
        return (self.data[0] * self.data[0]
            + self.data[1] * self.data[1]
            + self.data[2] * self.data[2]);
    }

    // returns 1 / length
    #[inline]
    pub fn inv_sqrt_len(&self) -> f32 {
        let len_squared =
            self.data[0] * self.data[0] + self.data[1] * self.data[1] + self.data[2] * self.data[2];
        return 1.0 / len_squared.sqrt();
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        return Vec3::multiply_by_f32(*self, self.inv_sqrt_len());
    }
}

// overloaded operators
impl std::ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::add(self, rhs)
    }
}

impl std::ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3::subtract(self, rhs)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;
    #[inline]
    fn neg(self) -> Self::Output {
        Vec3::new([-self.data[0], -self.data[1], -self.data[2]])
    }
}

/// by-component multiplication
impl std::ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::multiply_components(self, rhs)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Vec3::multiply_by_f32(self, rhs)
    }
}
impl std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::multiply_by_f32(rhs, self)
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        return Vec3::divide_by_f32(self, rhs);
    }
}

#[derive(Clone, Copy)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { direction, origin }
    }

    pub fn origin(&self) -> Vec3 {
        return self.origin;
    }

    pub fn direction(&self) -> Vec3 {
        return self.direction;
    }

    pub fn point_at_parameter(&self, t: f32) -> Vec3 {
        return Vec3::add(self.origin, Vec3::multiply_by_f32(self.direction, t));
    }
}
