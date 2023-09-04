use std::{arch::x86_64::*, fmt::Debug, mem::MaybeUninit};

use super::Mat44;

#[derive(Clone, Copy)]
pub union Vec3 {
    pub(super) data: [f32; 4],
    pub(super) data_vectorized: __m128,
}

impl Debug for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec3<{};{};{}>", self.x(), self.y(), self.z())
    }
}

impl Vec3 {
    pub const ZERO: Self = Vec3::new([0.0, 0.0, 0.0]);
    pub const ONE: Self = Vec3::new([1.0, 1.0, 1.0]);
    pub const BACK: Self = Vec3::new([0.0, 0.0, -1.0]);
    pub const UP: Self = Vec3::new([0.0, 1.0, 0.0]);
    pub const X_AXIS: Self = Vec3::new([1.0, 0.0, 0.0]);
    pub const Y_AXIS: Self = Vec3::new([0.0, 1.0, 0.0]);
    pub const Z_AXIS: Self = Vec3::new([0.0, 0.0, 1.0]);

    pub const fn new(data: [f32; 3]) -> Self {
        let data = [data[0], data[1], data[2], 0.0];
        Self::from_f32(data)
    }

    #[inline]
    #[must_use]
    pub const fn from_f32(data: [f32; 4]) -> Self {
        Self { data }
    }

    #[inline]
    #[must_use]
    pub const fn from_f32_3(data: [f32; 3], data3: f32) -> Self {
        Self {
            data: [data[0], data[1], data[2], data3]
        }
    }

    #[inline]
    #[must_use]
    pub const fn x(&self) -> f32 {
        unsafe { self.data[0] }
    }

    #[inline]
    #[must_use]
    pub const fn y(&self) -> f32 {
        unsafe { self.data[1] }
    }

    #[inline]
    #[must_use]
    pub const fn z(&self) -> f32 {
        unsafe { self.data[2] }
    }
    #[inline]
    #[must_use]
    pub const fn w(&self) -> f32 {
        unsafe { self.data[3] }
    }

    #[inline]
    fn uninit() -> Self {
        return unsafe { MaybeUninit::<Vec3>::uninit().assume_init() };
    }

    pub fn extract(&self) -> [f32; 4] {
        unsafe { self.data }
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
    pub fn divided_by_w(&self) -> Self {
        if self.w() == 0.0 {
            self.clone()
        }
        else {
            Vec3::divide_by_f32(*self, self.w())
        }
    }

    #[inline]
    #[must_use]
    pub fn as_point(&self) -> Self {
        Self::from_f32([self.x(), self.y(), self.z(), 1.0])
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

    #[cfg(target_feature = "sse")]
    pub const fn from_sse(sse: __m128) -> Self {
        Self {
            data_vectorized: sse,
        }
    }

    #[inline]
    pub fn abs(&self) -> Self {
        unsafe { return Self::new([self.x().abs(), self.y().abs(), self.z().abs()]) }
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
            }
        }
    }

    #[inline]
    pub fn flip_y(&self) -> Self {
        return Self::new([self.x(), -self.y(), self.z()]);
    }

    #[inline]
    pub fn sum_components(&self) -> f32 {
        #[cfg(target_feature = "sse")]
        unsafe {
            let d = self.data_vectorized;
            let d = _mm_hadd_ps(d, d);
            let d = _mm_hadd_ps(d, d);

            let ptr = (&d) as *const __m128 as *const f32;
            return *ptr.add(0);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            self.x() + self.y() + self.z()
        }
    }

    #[inline]
    pub fn luminosity(&self) -> f32 {
        (*self * Self::from_f32([0.2126, 0.7152, 0.0722, 0.0])).sum_components()
    }

    #[inline]
    pub fn saturate(&self) -> Vec3 {
        self.clamp(f32::EPSILON, 1.0 - f32::EPSILON)
    }

    #[inline]
    pub fn calculate_normal_from_points(p0: Vec3, p1: Vec3, p2: Vec3) -> Vec3 {
        
        let vertex0: Vec3 = p0;
        let vertex1: Vec3 = p1;
        let vertex2: Vec3 = p2;
        let edge1 = vertex1 - vertex0;
        let edge2 = vertex2 - vertex0;

        let geometry_normal = Vec3::cross(edge1, edge2).normalized();
        geometry_normal
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

impl std::ops::SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self = Self::subtract(*self, rhs);
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

impl std::ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self::multiply_by_f32(*self, rhs);
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        return Vec3::divide_by_f32(self, rhs);
    }
}

impl std::ops::Mul<&Mat44> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: &Mat44) -> Self::Output {
        Mat44::transform_point(&rhs, self)
    }
}

impl From<__m128> for Vec3 {
    fn from(value: __m128) -> Self {
        Self {
            data_vectorized: value
        }
    }
}