use core::arch::x86_64::*;
use std::{mem::MaybeUninit, u128, intrinsics::fabsf32};

use crate::tracing::MAX_BOUNCES;

pub mod random;

const unsafe fn make_m128(x: f32, y: f32, z: f32, w: f32) -> __m128 {
    unsafe {
        let x: u128 = std::mem::transmute::<f32, u32>(x) as u128;
        let y: u128 = std::mem::transmute::<f32, u32>(y) as u128;
        let z: u128 = std::mem::transmute::<f32, u32>(z) as u128;
        let w: u128 = std::mem::transmute::<f32, u32>(w) as u128;

        let output: u128 = x << 0 | y << 32 | z << 64 | w << 96;
        let output: __m128 = std::mem::transmute(output);
        return output;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    #[cfg(not(target_feature = "sse"))]
    data: [f32; 4],
    #[cfg(target_feature = "sse")]
    data_vectorized: __m128,
}

impl Vec3 {
    pub const ZERO: Self = Vec3::new([0.0, 0.0, 0.0]);
    pub const ONE: Self = Vec3::new([1.0, 1.0, 1.0]);
    pub const BACK: Self = Vec3::new([0.0, 0.0, -1.0]);
    pub const UP: Self = Vec3::new([0.0, 1.0, 0.0]);

    pub const fn new(data: [f32; 3]) -> Self {
        let data = [data[0], data[1], data[2], 0.0];
        Self::from_f32(data)
    }

    #[inline]
    #[must_use]
    pub const fn from_f32(data: [f32; 4]) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            let data_vectorized: __m128 = make_m128(data[0], data[1], data[2], data[3]);

            Self { data_vectorized }
        }
        #[cfg(not(target_feature = "sse"))]
        {
            Self { data }
        }
    }

    #[inline]
    #[must_use]
    pub const fn x(&self) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            let ptr = (&self.data_vectorized) as *const __m128 as *const f32;
            return *ptr.add(0);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            data[0]
        }
    }

    #[inline]
    #[must_use]
    pub const fn y(&self) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            let ptr = (&self.data_vectorized) as *const __m128 as *const f32;
            let ptr = ptr.add(1);
            return *ptr;
        }
        #[cfg(not(target_feature = "sse"))]
        {
            data[1]
        }
    }

    #[inline]
    #[must_use]
    pub const fn z(&self) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            let ptr = (&self.data_vectorized) as *const __m128 as *const f32;
            let ptr = ptr.add(2);
            return *ptr;
        }
        #[cfg(not(target_feature = "sse"))]
        {
            data[2]
        }
    }

    #[inline]
    fn uninit() -> Self {
        return unsafe { MaybeUninit::<Vec3>::uninit().assume_init() };
    }

    pub fn extract(&self) -> [f32; 4] {
        unsafe {
            [
                f32::from_bits(_mm_extract_ps::<0>(self.data_vectorized) as u32),
                f32::from_bits(_mm_extract_ps::<1>(self.data_vectorized) as u32),
                f32::from_bits(_mm_extract_ps::<2>(self.data_vectorized) as u32),
                f32::from_bits(_mm_extract_ps::<3>(self.data_vectorized) as u32),
            ]
        }
    }

    #[inline]
    #[must_use]
    pub fn add(left: Vec3, right: Vec3) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            return Self {
                data_vectorized: _mm_add_ps(left.data_vectorized, right.data_vectorized),
            };
        }

        #[cfg(not(target_feature = "sse"))]
        {
            return Self::new([
                left.x() + right.x(),
                left.y() + right.y(),
                left.z() + right.z(),
            ]);
        }
    }

    #[inline]
    #[must_use]
    pub fn subtract(left: Vec3, right: Vec3) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            return Self {
                data_vectorized: _mm_sub_ps(left.data_vectorized, right.data_vectorized),
            };
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return Self::new([
                left.x() - right.x(),
                left.y() - right.y(),
                left.z() - right.z(),
            ]);
        }
    }

    #[inline]
    #[must_use]
    pub fn multiply_components(left: Vec3, right: Vec3) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            return Self {
                data_vectorized: _mm_mul_ps(left.data_vectorized, right.data_vectorized),
            };
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return Self::new([
                left.x() * right.x(),
                left.y() * right.y(),
                left.z() * right.z(),
            ]);
        }
    }

    #[inline]
    #[must_use]
    pub fn multiply_by_f32(left: Vec3, right: f32) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            let right = _mm_set1_ps(right);
            return Self {
                data_vectorized: _mm_mul_ps(left.data_vectorized, right),
            };
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return Self::new([left.x() * right, left.y() * right, left.z() * right]);
        }
    }

    #[inline]
    #[must_use]
    pub fn divide_by_f32(left: Vec3, right: f32) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            let right = _mm_set1_ps(right);
            return Self {
                data_vectorized: _mm_div_ps(left.data_vectorized, right),
            };
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return Self::new([left.x() / right, left.y() / right, left.z() / right]);
        }
    }

    const IMM8: i32 = 1 << 0 | 1 << 1 | 1 << 2 | 0 << 3 | 1 << 4 | 1 << 5 | 1 << 6 | 0 << 7;

    #[inline]
    #[must_use]
    #[cfg(target_feature = "sse")]
    fn m128_dot(left: __m128, right: __m128) -> f32 {
        unsafe {
            let dp = _mm_dp_ps::<{ Self::IMM8 }>(left, right);
            return _mm_cvtss_f32(dp);
        }
    }

    #[inline]
    #[must_use]
    pub fn dot(left: Vec3, right: Vec3) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            return Self::m128_dot(left.data_vectorized, right.data_vectorized);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return (left.x() * right.x()) + (left.y() * right.y()) + (left.z() * right.z());
        }
    }

    #[inline]
    #[must_use]
    pub fn cross(left: Vec3, right: Vec3) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            let a = left.data_vectorized;
            let b = right.data_vectorized;

            let a_yzx: __m128 = _mm_shuffle_ps(a, a, _MM_SHUFFLE(3, 0, 2, 1));
            let b_yzx: __m128 = _mm_shuffle_ps(b, b, _MM_SHUFFLE(3, 0, 2, 1));
            let c: __m128 = _mm_sub_ps(_mm_mul_ps(a, b_yzx), _mm_mul_ps(a_yzx, b));
            let data_vectorized = _mm_shuffle_ps(c, c, _MM_SHUFFLE(3, 0, 2, 1));
            return Self { data_vectorized };
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return Self::new([
                left.y() * right.z() - left.z() * right.y(),
                -(left.x() * right.z() - left.z() * right.x()),
                left.x() * right.y() - left.y() * right.x(),
            ]);
        }
    }

    pub fn cross2d_z(left: Vec3, right: Vec3) -> f32 {
        Self::cross(left, right).z()
    }

    #[inline]
    #[must_use]
    pub fn length(&self) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            let squared_length =
                _mm_dp_ps::<{ Self::IMM8 }>(self.data_vectorized, self.data_vectorized);
            let one = _mm_set1_ps(1.0);
            let inverted_length = _mm_rsqrt_ss(squared_length);
            let length = _mm_div_ss(one, inverted_length);
            return _mm_cvtss_f32(length);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return self.squared_length().sqrt();
        }
    }

    #[inline]
    #[must_use]
    pub fn squared_length(&self) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            return Self::m128_dot(self.data_vectorized, self.data_vectorized);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return (self.x() * self.x() + self.y() * self.y() + self.z() * self.z());
        }
    }

    #[inline]
    #[must_use]
    pub fn normalized(&self) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            let squared_length =
                _mm_dp_ps::<{ Self::IMM8 }>(self.data_vectorized, self.data_vectorized);
            let packed_length = _mm_rsqrt_ss(squared_length);
            let all_length: __m128 =
                _mm_shuffle_ps(packed_length, packed_length, _MM_SHUFFLE(0, 0, 0, 0));
            let normalized = _mm_mul_ps(self.data_vectorized, all_length);
            let result = Self {
                data_vectorized: normalized,
            };
            return result;
        }
        #[cfg(not(target_feature = "sse"))]
        {
            let result = Vec3::divide_by_f32(*self, self.length());
            return result;
        }
    }

    #[inline]
    #[must_use]
    pub fn clamp(&self, min: f32, max: f32) -> Self {
        return Vec3::new([
            self.x().clamp(min, max),
            self.y().clamp(min, max),
            self.z().clamp(min, max),
        ]);
    }

    /// Gamma 2.0 -> pow(x, 1.0 / 2.0) -> sqrt(x)
    #[inline]
    #[must_use]
    pub fn gamma_correct_2(&self) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            Self {
                data_vectorized: _mm_sqrt_ps(self.data_vectorized),
            }
        }
        #[cfg(not(target_feature = "sse"))]
        {
            return Vec3::new([self.x().sqrt(), self.y().sqrt(), self.z().sqrt()]);
        }
    }

    #[inline]
    #[must_use]
    pub fn lerp(left: Vec3, right: Vec3, t: f32) -> Self {
        // #[cfg(target_feature = "sse")]
        // unsafe {
        //     let t = t.clamp(0.0, 1.0);

        //     let t = _mm_set1_ps(t);

        //     let diff = _mm_sub_ps(left.data_vectorized, right.data_vectorized);
        //     let diff_scaled = _mm_mul_ps(diff, t);
        //     let result = _mm_add_ps(left.data_vectorized, diff_scaled);
        //     Self {
        //         data_vectorized: result,
        //     }
        // }
        // #[cfg(not(target_feature = "sse"))]
        {
            let result = Vec3::new([
                left.x() + (right.x() - left.x()) * t,
                left.y() + (right.y() - left.y()) * t,
                left.z() + (right.z() - left.z()) * t,
            ]);
            return result;
        }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new([r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0])
    }

    pub const fn from_packed_u32_rgb(packed: u32) -> Self {
        unsafe {
            let r = (packed >> 0) & 0x000000FF;
            let g = (packed >> 8) & 0x000000FF;
            let b = (packed >> 16) & 0x000000FF;
            let a = (packed >> 24) & 0x000000FF;
            Self::from_rgb(r as u8, g as u8, b as u8)
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        unsafe 
        {
            return Self::new([
                self.x().abs(),
                self.y().abs(),
                self.z().abs()
            ])
        }
    }

    #[inline]
    pub fn index_unchecked(&self, index: usize) -> f32 {
        // unoptimized
        match index {
            0 => self.x(),
            1 => self.y(),
            2 => self.z(),
            _ => {
                panic!("index_unchecked pee pee poo poo")
            },
        }
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

impl std::ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = Vec3::add(*self, rhs)
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
        Vec3::new([-self.x(), -self.y(), -self.z()])
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

pub fn reflect(vector: Vec3, normal: Vec3) -> Vec3 {
    return vector - 2.0 * Vec3::dot(vector, normal) * normal;
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

pub struct RayBounce {
    pub ray: Ray,
    pub bounces: i32,
    pub multiplier: f32,
}

impl RayBounce {
    pub fn new(ray: Ray) -> Self {
        Self {
            ray,
            bounces: MAX_BOUNCES,
            multiplier: 1.0,
        }
    }
}

impl Into<RayBounce> for Ray {
    fn into(self) -> RayBounce {
        RayBounce::new(self)
    }
}
