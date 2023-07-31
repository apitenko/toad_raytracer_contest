use core::arch::x86_64::*;
use std::{mem::MaybeUninit, u128};

pub mod random;

const unsafe fn make_m128(a: f32, b: f32, c: f32, d: f32) -> __m128 {
    unsafe {
        let a: u128 = std::mem::transmute::<f32, u32>(a) as u128;
        let b: u128 = std::mem::transmute::<f32, u32>(b) as u128;
        let c: u128 = std::mem::transmute::<f32, u32>(c) as u128;
        let d: u128 = std::mem::transmute::<f32, u32>(d) as u128;

        let output: u128 = a << 0 | b << 32 | c << 64 | d << 96;
        let output: __m128 = std::mem::transmute(output);
        return output;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    data: [f32; 4],
    #[cfg(target_feature = "avx")]
    data_vectorized: __m128,
}

impl Vec3 {
    pub const ZERO: Self = Vec3::new([0.0, 0.0, 0.0]);
    pub const ONE: Self = Vec3::new([1.0, 1.0, 1.0]);
    pub const BACK: Self = Vec3::new([0.0, 0.0, -1.0]);

    #[inline]
    #[must_use]
    pub const fn new(data: [f32; 3]) -> Self {
        let data = [data[0], data[1], data[2], 0.0];

        #[cfg(target_feature = "avx")]
        unsafe {
            let data_vectorized: __m128 = make_m128(data[0], data[1], data[2], data[3]);

            Self {
                data,
                data_vectorized,
            }
        }
        #[cfg(not(target_feature = "avx"))]
        {
            Self { data }
        }
    }

    #[inline]
    #[must_use]
    pub fn x(&self) -> f32 {
        return self.data[0];
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> f32 {
        return self.data[1];
    }

    #[inline]
    #[must_use]
    pub fn z(&self) -> f32 {
        return self.data[2];
    }

    #[inline]
    fn uninit() -> Self {
        return unsafe { MaybeUninit::<Vec3>::uninit().assume_init() };
    }

    fn extract(packed: __m128) -> [f32; 4] {
        unsafe {
            [
                f32::from_bits(_mm_extract_ps::<0>(packed) as u32),
                f32::from_bits(_mm_extract_ps::<1>(packed) as u32),
                f32::from_bits(_mm_extract_ps::<2>(packed) as u32),
                f32::from_bits(_mm_extract_ps::<3>(packed) as u32),
            ]
        }
    }

    #[inline]
    #[must_use]
    pub fn add(left: Vec3, right: Vec3) -> Self {
        #[cfg(target_feature = "avx")]
        unsafe {
            let data_vectorized = _mm_add_ps(left.data_vectorized, right.data_vectorized);

            return Self {
                data_vectorized: _mm_add_ps(left.data_vectorized, right.data_vectorized),
                data: Self::extract(data_vectorized),
            };
        }

        #[cfg(not(target_feature = "avx"))]
        {
            return Self::new([
                left.data[0] + right.data[0],
                left.data[1] + right.data[1],
                left.data[2] + right.data[2],
            ]);
        }
    }

    #[inline]
    #[must_use]
    pub fn subtract(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[0] - right.data[0],
            left.data[1] - right.data[1],
            left.data[2] - right.data[2],
        ]);
    }

    #[inline]
    #[must_use]
    pub fn multiply_components(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[0] * right.data[0],
            left.data[1] * right.data[1],
            left.data[2] * right.data[2],
        ]);
    }

    #[inline]
    #[must_use]
    pub fn multiply_by_f32(left: Vec3, right: f32) -> Self {
        return Self::new([
            left.data[0] * right,
            left.data[1] * right,
            left.data[2] * right,
        ]);
    }

    #[inline]
    #[must_use]
    pub fn divide_by_f32(left: Vec3, right: f32) -> Self {
        return Self::new([
            left.data[0] / right,
            left.data[1] / right,
            left.data[2] / right,
        ]);
    }

    #[inline]
    #[must_use]
    pub fn dot(left: Vec3, right: Vec3) -> f32 {
        return (left.data[0] * right.data[0])
            + (left.data[1] * right.data[1])
            + (left.data[2] * right.data[2]);
    }

    #[inline]
    #[must_use]
    pub fn cross(left: Vec3, right: Vec3) -> Self {
        return Self::new([
            left.data[1] * right.data[2] - left.data[2] * right.data[1],
            -(left.data[0] * right.data[2] - left.data[2] * right.data[0]),
            left.data[0] * right.data[1] - left.data[1] * right.data[0],
        ]);
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> f32 {
        return self.squared_length().sqrt();
    }

    #[inline]
    #[must_use]
    pub fn squared_length(&self) -> f32 {
        return (self.data[0] * self.data[0]
            + self.data[1] * self.data[1]
            + self.data[2] * self.data[2]);
    }

    // returns 1 / length
    #[inline]
    #[must_use]
    pub fn inv_sqrt_len(&self) -> f32 {
        let len_squared =
            self.data[0] * self.data[0] + self.data[1] * self.data[1] + self.data[2] * self.data[2];
        return 1.0 / len_squared.sqrt();
    }

    #[inline]
    #[must_use]
    pub fn normalized(&self) -> Self {
        return Vec3::multiply_by_f32(*self, self.inv_sqrt_len());
    }

    #[inline]
    #[must_use]
    pub fn clamp(&self, min: f32, max: f32) -> Self {
        return Vec3::new([
            self.data[0].clamp(min, max),
            self.data[1].clamp(min, max),
            self.data[2].clamp(min, max),
        ]);
    }

    /// Gamma 2.0 -> pow(x, 1.0 / 2.0) -> sqrt(x)
    #[inline]
    #[must_use]
    pub fn gamma_correct_2(&self) -> Self {
        return Vec3::new([
            self.data[0].sqrt(),
            self.data[1].sqrt(),
            self.data[2].sqrt(),
        ]);
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
    max_distance: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, max_distance: f32) -> Self {
        Self {
            direction,
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
