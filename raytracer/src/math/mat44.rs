use std::{arch::x86_64::*, f32::consts::PI, mem::MaybeUninit};

use super::Vec3;

#[derive(Clone, Copy)]
pub union Mat44 {
    m: [[f32; 4]; 4],
    row: [__m128; 4],
}

impl Mat44 {
    pub const IDENTITY: Self = Self::new([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    pub const fn new(m: [[f32; 4]; 4]) -> Self {
        Self { m }
    }

    fn from_m128_rows(row: [__m128; 4]) -> Self {
        Self { row }
    }

    // row-major
    pub fn from_translation(translation: [f32; 3]) -> Self {
        let t = translation;
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [t[0], t[1], t[2], 1.0],
            ],
        }
    }

    pub fn from_rotation_quaternion(rotation: [f32; 4]) -> Self {
        let mut qx = rotation[0];
        let mut qy = rotation[1];
        let mut qz = rotation[2];
        let mut qw = rotation[3];

        let n: f32 = 1.0 / (qx * qx + qy * qy + qz * qz + qw * qw).sqrt();
        qx *= n;
        qy *= n;
        qz *= n;
        qw *= n;

        let m = [
            [
                1.0 - 2.0 * qy * qy - 2.0 * qz * qz,
                2.0 * qx * qy - 2.0 * qz * qw,
                2.0 * qx * qz + 2.0 * qy * qw,
                0.0,
            ],
            [
                2.0 * qx * qy + 2.0 * qz * qw,
                1.0 - 2.0 * qx * qx - 2.0 * qz * qz,
                2.0 * qy * qz - 2.0 * qx * qw,
                0.0,
            ],
            [
                2.0 * qx * qz - 2.0 * qy * qw,
                2.0 * qy * qz + 2.0 * qx * qw,
                1.0 - 2.0 * qx * qx - 2.0 * qy * qy,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ];

        Self { m }.transposed() // transpose for right-handedness
    }

    pub fn from_scale(scale: [f32; 3]) -> Self {
        Self {
            m: [
                [scale[0], 0.0, 0.0, 0.0],
                [0.0, scale[1], 0.0, 0.0],
                [0.0, 0.0, scale[2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_decomposed(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        Self::from_scale(scale)
        * Self::from_rotation_quaternion(rotation)
        * Self::from_translation(translation)
    }

    pub fn from_4x4(matrix: [[f32; 4]; 4]) -> Self {
        Self { m: matrix }
    }

    // row-major
    // pub fn from_perspective_lh(yfov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
    //     let (sin_fov, cos_fov) = (0.5 * yfov).sin_cos();
    //     let h = cos_fov / sin_fov;
    //     let w = h / aspect_ratio;
    //     let r = z_far / (z_far - z_near);
    //     let m = [
    //         [w, 0.0, 0.0, 0.0],
    //         [0.0, h, 0.0, 0.0],
    //         [0.0, 0.0, r, -r * z_near],
    //         [0.0, 0.0, 1.0, 0.0],
    //     ];
    //     Self { m }
    // }

    // row-major
    pub fn from_perspective_rh(yfov: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        let (sin_fov, cos_fov) = (0.5 * yfov).sin_cos();
        let h = cos_fov / sin_fov;
        let w = h * aspect_ratio;
        let r = z_far / (z_near - z_far);
        let m = [
            [w, 0.0, 0.0, 0.0],
            [0.0, h, 0.0, 0.0],
            [0.0, 0.0, r, r * z_near],
            [0.0, 0.0, -1.0, 0.0],
        ];
        Self { m }
    }

    pub fn from_perspective_infinite(yfov: f32, aspect_ratio: f32, near: f32) -> Self {
        todo!()
    }

    pub fn from_orthographic(xmag: f32, ymag: f32, near: f32, far: f32) -> Self {
        todo!()
        // let rcp_width = 1.0 / (right - left);
        // let rcp_height = 1.0 / (top - bottom);
        // let r = 1.0 / (near - far);
        // Self {
        //     m: [
        //         [rcp_width + rcp_width, 0.0, 0.0, 0.0],
        //         [0.0, rcp_height + rcp_height, 0.0, 0.0],
        //         [0.0, 0.0, r, 0.0],
        //         [-(left + right) * rcp_width, -(top + bottom) * rcp_height, r * near, 1.0],
        //     ]
        // }
    }

    #[inline]
    pub fn multiply(left: &Self, right: &Self) -> Self {
        #[cfg(target_feature = "avx")]
        unsafe {
            matmult_AVX_8(left, right)
        }
        #[cfg(not(target_feature = "avx"))]
        {
            panic!("Not supported");
        }
    }

    pub fn multiply_vec(&self, vector: Vec3) -> Vec3 {
        
        #[cfg(target_feature = "sse")]
        unsafe {
            let vec_x: __m128 = _mm_permute_ps(vector.data_vectorized, 0x00);
            let vec_y: __m128 = _mm_permute_ps(vector.data_vectorized, 0x55);
            let vec_z: __m128 = _mm_permute_ps(vector.data_vectorized, 0xAA);
            let vec_w: __m128 = _mm_permute_ps(vector.data_vectorized, 0xFF);

            // assume mat4_1, mat4_2, mat4_3, mat4_4 are matrix's components (I think rows)
            let res0: __m128 = _mm_mul_ps(vec_x, self.row[0]);
            let res1: __m128 = _mm_fmadd_ps(vec_y, self.row[1], res0);
            let res2: __m128 = _mm_fmadd_ps(vec_z, self.row[2], res1);
            let res3: __m128 = _mm_fmadd_ps(vec_w, self.row[3], res2);
            return Vec3::from_sse(res3);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            panic!("Not supported");
        }
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.multiply_vec(point).divided_by_w().as_point()
    }

    // ! probably won't work for non-transform matrices
    pub fn inverse(&self) -> Self {
        unsafe { GetTransformInverse_glam(&self) }
    }
    pub fn transposed(&self) -> Self {
        #[cfg(target_feature = "sse")]
        unsafe {
            let copy = self.clone();

            let [mut copy_row0, mut copy_row1, mut copy_row2, mut copy_row3] = copy.row;

            _MM_TRANSPOSE4_PS(
                &mut copy_row0,
                &mut copy_row1,
                &mut copy_row2,
                &mut copy_row3,
            );
            return Mat44::from_m128_rows([copy_row0, copy_row1, copy_row2, copy_row3]);
        }
        #[cfg(not(target_feature = "sse"))]
        {
            panic!("Not supported");
        }
    }
}

impl std::ops::Mul<Mat44> for Mat44 {
    type Output = Mat44;

    #[inline]
    fn mul(self, rhs: Mat44) -> Self::Output {
        Mat44::multiply(&self, &rhs)
    }
}

impl std::ops::Mul<&Mat44> for Mat44 {
    type Output = Mat44;

    #[inline]
    fn mul(self, rhs: &Mat44) -> Self::Output {
        Mat44::multiply(&self, rhs)
    }
}

impl std::ops::Mul<Mat44> for &Mat44 {
    type Output = Mat44;

    #[inline]
    fn mul(self, rhs: Mat44) -> Self::Output {
        Mat44::multiply(&self, &rhs)
    }
}

// Don't implement this
impl !std::ops::Mul<Mat44> for Vec3 {}

impl std::ops::Mul<Vec3> for Mat44 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::multiply_vec(&self, rhs)
    }
}

// helpers

// dual linear combination using AVX instructions on YMM regs
#[inline]
unsafe fn twolincomb_AVX_8(A01: __m256, B: &Mat44) -> __m256 {
    let result = _mm256_mul_ps(
        _mm256_shuffle_ps(A01, A01, 0x00),
        _mm256_broadcast_ps(&B.row[0]),
    );
    let result = _mm256_add_ps(
        result,
        _mm256_mul_ps(
            _mm256_shuffle_ps(A01, A01, 0x55),
            _mm256_broadcast_ps(&B.row[1]),
        ),
    );
    let result = _mm256_add_ps(
        result,
        _mm256_mul_ps(
            _mm256_shuffle_ps(A01, A01, 0xaa),
            _mm256_broadcast_ps(&B.row[2]),
        ),
    );
    let result = _mm256_add_ps(
        result,
        _mm256_mul_ps(
            _mm256_shuffle_ps(A01, A01, 0xff),
            _mm256_broadcast_ps(&B.row[3]),
        ),
    );
    return result;
}

// this should be noticeably faster with actual 256-bit wide vector units (Intel);
// not sure about double-pumped 128-bit (AMD), would need to check.
#[inline]
unsafe fn matmult_AVX_8(A: &Mat44, B: &Mat44) -> Mat44 {
    _mm256_zeroupper();
    let A01: __m256 = _mm256_loadu_ps(&A.m[0][0]);
    let A23: __m256 = _mm256_loadu_ps(&A.m[2][0]);

    let out01x: __m256 = twolincomb_AVX_8(A01, B);
    let out23x: __m256 = twolincomb_AVX_8(A23, B);

    let mut out: Mat44 = MaybeUninit::uninit().assume_init();
    _mm256_storeu_ps(&mut out.m[0][0] as *mut f32, out01x);
    _mm256_storeu_ps(&mut out.m[2][0] as *mut f32, out23x);
    return out;
}

const fn MakeShuffleMask<const x: i32, const y: i32, const z: i32, const w: i32>() -> i32 {
    x | (y << 2) | (z << 4) | (w << 6)
}

// vec(0, 1, 2, 3) -> (vec[x], vec[y], vec[z], vec[w])
unsafe fn VecSwizzleMask<const mask: i32>(vec: __m128) -> __m128 {
    _mm_castsi128_ps(_mm_shuffle_epi32(_mm_castps_si128(vec), mask))
}

unsafe fn VecSwizzle<const x: i32, const y: i32, const z: i32, const w: i32>(vec: __m128) -> __m128
where
    [(); MakeShuffleMask::<{ x }, { y }, { z }, { w }>() as usize]:,
{
    VecSwizzleMask::<{ MakeShuffleMask::<{ x }, { y }, { z }, { w }>() }>(vec)
}

unsafe fn VecSwizzle1<const x: i32>(vec: __m128) -> __m128
where
    [(); MakeShuffleMask::<{ x }, { x }, { x }, { x }>() as usize]:,
{
    VecSwizzleMask::<{ MakeShuffleMask::<{ x }, { x }, { x }, { x }>() }>(vec)
}

// special swizzle
unsafe fn VecSwizzle_0022(vec: __m128) -> __m128 {
    _mm_moveldup_ps(vec)
}

unsafe fn VecSwizzle_1133(vec: __m128) -> __m128 {
    _mm_movehdup_ps(vec)
}

// return (vec1[x], vec1[y], vec2[z], vec2[w])
unsafe fn VecShuffle<const x: i32, const y: i32, const z: i32, const w: i32>(
    vec1: __m128,
    vec2: __m128,
) -> __m128
where
    [(); MakeShuffleMask::<{ x }, { y }, { z }, { w }>() as usize]:,
{
    _mm_shuffle_ps::<{ MakeShuffleMask::<{ x }, { y }, { z }, { w }>() }>(vec1, vec2)
}

// special shuffle
unsafe fn VecShuffle_0101(vec1: __m128, vec2: __m128) -> __m128 {
    _mm_movelh_ps(vec1, vec2)
}
unsafe fn VecShuffle_2323(vec1: __m128, vec2: __m128) -> __m128 {
    _mm_movehl_ps(vec2, vec1)
}

unsafe fn Mat2Mul(vec1: __m128, vec2: __m128) -> __m128 {
    return _mm_add_ps(
        _mm_mul_ps(vec1, VecSwizzle::<0, 0, 3, 3>(vec2)),
        _mm_mul_ps(
            VecSwizzle::<2, 3, 0, 1>(vec1),
            VecSwizzle::<1, 1, 2, 2>(vec2),
        ),
    );
}
// 2x2 column major Matrix adjugate multiply (A#)*B
unsafe fn Mat2AdjMul(vec1: __m128, vec2: __m128) -> __m128 {
    return _mm_sub_ps(
        _mm_mul_ps(VecSwizzle::<3, 0, 3, 0>(vec1), vec2),
        _mm_mul_ps(
            VecSwizzle::<2, 1, 2, 1>(vec1),
            VecSwizzle::<1, 0, 3, 2>(vec2),
        ),
    );
}
// 2x2 column major Matrix multiply adjugate A*(B#)
unsafe fn Mat2MulAdj(vec1: __m128, vec2: __m128) -> __m128 {
    return _mm_sub_ps(
        _mm_mul_ps(vec1, VecSwizzle::<3, 3, 0, 0>(vec2)),
        _mm_mul_ps(
            VecSwizzle::<2, 3, 0, 1>(vec1),
            VecSwizzle::<1, 1, 2, 2>(vec2),
        ),
    );
}

const SMALL_NUMBER: f32 = 1e-8;

// Requires this matrix to be transform matrix
unsafe fn GetTransformInverse(inM: &Mat44) -> Mat44 {
    let mut r = Mat44::IDENTITY;

    // transpose 3x3, we know m03 = m13 = m23 = 0
    let t0 = VecShuffle_0101(inM.row[0], inM.row[1]); // 00, 01, 10, 11
    let t1 = VecShuffle_2323(inM.row[0], inM.row[1]); // 02, 03, 12, 13
    r.row[0] = VecShuffle::<0, 2, 0, 3>(t0, inM.row[2]); // 00, 10, 20, 23(=0)
    r.row[1] = VecShuffle::<1, 3, 1, 3>(t0, inM.row[2]); // 01, 11, 21, 23(=0)
    r.row[2] = VecShuffle::<0, 2, 2, 3>(t1, inM.row[2]); // 02, 12, 22, 23(=0)

    // (SizeSqr(row[0]), SizeSqr(row[1]), SizeSqr(row[2]), 0)
    let mut sizeSqr: __m128;
    sizeSqr = _mm_mul_ps(r.row[0], r.row[0]);
    sizeSqr = _mm_add_ps(sizeSqr, _mm_mul_ps(r.row[1], r.row[1]));
    sizeSqr = _mm_add_ps(sizeSqr, _mm_mul_ps(r.row[2], r.row[2]));

    // optional test to avoid divide by 0
    let one = _mm_set1_ps(1.0);
    // for each component, if(sizeSqr < SMALL_NUMBER) sizeSqr = 1;
    let rSizeSqr = _mm_blendv_ps(
        _mm_div_ps(one, sizeSqr),
        one,
        _mm_cmplt_ps(sizeSqr, _mm_set1_ps(SMALL_NUMBER)),
    );

    r.row[0] = _mm_mul_ps(r.row[0], rSizeSqr);
    r.row[1] = _mm_mul_ps(r.row[1], rSizeSqr);
    r.row[2] = _mm_mul_ps(r.row[2], rSizeSqr);

    // last line
    r.row[3] = _mm_mul_ps(r.row[0], VecSwizzle1::<0>(inM.row[3]));
    r.row[3] = _mm_add_ps(r.row[3], _mm_mul_ps(r.row[1], VecSwizzle1::<1>(inM.row[3])));
    r.row[3] = _mm_add_ps(r.row[3], _mm_mul_ps(r.row[2], VecSwizzle1::<2>(inM.row[3])));
    r.row[3] = _mm_sub_ps(_mm_setr_ps(0.0, 0.0, 0.0, 1.0), r.row[3]);

    return r;
}

// row major
fn GetTransformInverse_glam(inM: &Mat44) -> Mat44 {
    unsafe {
        let inM = inM.transposed(); // convert to column-major
        let w_axis = inM.row[3];
        let z_axis = inM.row[2];
        let y_axis = inM.row[1];
        let x_axis = inM.row[0];

        // Based on https://github.com/g-truc/glm `glm_mat4_inverse`
        let fac0 = {
            let swp0a = _mm_shuffle_ps(w_axis, z_axis, 0b11_11_11_11);
            let swp0b = _mm_shuffle_ps(w_axis, z_axis, 0b10_10_10_10);

            let swp00 = _mm_shuffle_ps(z_axis, y_axis, 0b10_10_10_10);
            let swp01 = _mm_shuffle_ps(swp0a, swp0a, 0b10_00_00_00);
            let swp02 = _mm_shuffle_ps(swp0b, swp0b, 0b10_00_00_00);
            let swp03 = _mm_shuffle_ps(z_axis, y_axis, 0b11_11_11_11);

            let mul00 = _mm_mul_ps(swp00, swp01);
            let mul01 = _mm_mul_ps(swp02, swp03);
            _mm_sub_ps(mul00, mul01)
        };
        let fac1 = {
            let swp0a = _mm_shuffle_ps(w_axis, z_axis, 0b11_11_11_11);
            let swp0b = _mm_shuffle_ps(w_axis, z_axis, 0b01_01_01_01);

            let swp00 = _mm_shuffle_ps(z_axis, y_axis, 0b01_01_01_01);
            let swp01 = _mm_shuffle_ps(swp0a, swp0a, 0b10_00_00_00);
            let swp02 = _mm_shuffle_ps(swp0b, swp0b, 0b10_00_00_00);
            let swp03 = _mm_shuffle_ps(z_axis, y_axis, 0b11_11_11_11);

            let mul00 = _mm_mul_ps(swp00, swp01);
            let mul01 = _mm_mul_ps(swp02, swp03);
            _mm_sub_ps(mul00, mul01)
        };
        let fac2 = {
            let swp0a = _mm_shuffle_ps(w_axis, z_axis, 0b10_10_10_10);
            let swp0b = _mm_shuffle_ps(w_axis, z_axis, 0b01_01_01_01);

            let swp00 = _mm_shuffle_ps(z_axis, y_axis, 0b01_01_01_01);
            let swp01 = _mm_shuffle_ps(swp0a, swp0a, 0b10_00_00_00);
            let swp02 = _mm_shuffle_ps(swp0b, swp0b, 0b10_00_00_00);
            let swp03 = _mm_shuffle_ps(z_axis, y_axis, 0b10_10_10_10);

            let mul00 = _mm_mul_ps(swp00, swp01);
            let mul01 = _mm_mul_ps(swp02, swp03);
            _mm_sub_ps(mul00, mul01)
        };
        let fac3 = {
            let swp0a = _mm_shuffle_ps(w_axis, z_axis, 0b11_11_11_11);
            let swp0b = _mm_shuffle_ps(w_axis, z_axis, 0b00_00_00_00);

            let swp00 = _mm_shuffle_ps(z_axis, y_axis, 0b00_00_00_00);
            let swp01 = _mm_shuffle_ps(swp0a, swp0a, 0b10_00_00_00);
            let swp02 = _mm_shuffle_ps(swp0b, swp0b, 0b10_00_00_00);
            let swp03 = _mm_shuffle_ps(z_axis, y_axis, 0b11_11_11_11);

            let mul00 = _mm_mul_ps(swp00, swp01);
            let mul01 = _mm_mul_ps(swp02, swp03);
            _mm_sub_ps(mul00, mul01)
        };
        let fac4 = {
            let swp0a = _mm_shuffle_ps(w_axis, z_axis, 0b10_10_10_10);
            let swp0b = _mm_shuffle_ps(w_axis, z_axis, 0b00_00_00_00);

            let swp00 = _mm_shuffle_ps(z_axis, y_axis, 0b00_00_00_00);
            let swp01 = _mm_shuffle_ps(swp0a, swp0a, 0b10_00_00_00);
            let swp02 = _mm_shuffle_ps(swp0b, swp0b, 0b10_00_00_00);
            let swp03 = _mm_shuffle_ps(z_axis, y_axis, 0b10_10_10_10);

            let mul00 = _mm_mul_ps(swp00, swp01);
            let mul01 = _mm_mul_ps(swp02, swp03);
            _mm_sub_ps(mul00, mul01)
        };
        let fac5 = {
            let swp0a = _mm_shuffle_ps(w_axis, z_axis, 0b01_01_01_01);
            let swp0b = _mm_shuffle_ps(w_axis, z_axis, 0b00_00_00_00);

            let swp00 = _mm_shuffle_ps(z_axis, y_axis, 0b00_00_00_00);
            let swp01 = _mm_shuffle_ps(swp0a, swp0a, 0b10_00_00_00);
            let swp02 = _mm_shuffle_ps(swp0b, swp0b, 0b10_00_00_00);
            let swp03 = _mm_shuffle_ps(z_axis, y_axis, 0b01_01_01_01);

            let mul00 = _mm_mul_ps(swp00, swp01);
            let mul01 = _mm_mul_ps(swp02, swp03);
            _mm_sub_ps(mul00, mul01)
        };
        let sign_a = _mm_set_ps(1.0, -1.0, 1.0, -1.0);
        let sign_b = _mm_set_ps(-1.0, 1.0, -1.0, 1.0);

        let temp0 = _mm_shuffle_ps(y_axis, x_axis, 0b00_00_00_00);
        let vec0 = _mm_shuffle_ps(temp0, temp0, 0b10_10_10_00);

        let temp1 = _mm_shuffle_ps(y_axis, x_axis, 0b01_01_01_01);
        let vec1 = _mm_shuffle_ps(temp1, temp1, 0b10_10_10_00);

        let temp2 = _mm_shuffle_ps(y_axis, x_axis, 0b10_10_10_10);
        let vec2 = _mm_shuffle_ps(temp2, temp2, 0b10_10_10_00);

        let temp3 = _mm_shuffle_ps(y_axis, x_axis, 0b11_11_11_11);
        let vec3 = _mm_shuffle_ps(temp3, temp3, 0b10_10_10_00);

        let mul00 = _mm_mul_ps(vec1, fac0);
        let mul01 = _mm_mul_ps(vec2, fac1);
        let mul02 = _mm_mul_ps(vec3, fac2);
        let sub00 = _mm_sub_ps(mul00, mul01);
        let add00 = _mm_add_ps(sub00, mul02);
        let inv0 = _mm_mul_ps(sign_b, add00);

        let mul03 = _mm_mul_ps(vec0, fac0);
        let mul04 = _mm_mul_ps(vec2, fac3);
        let mul05 = _mm_mul_ps(vec3, fac4);
        let sub01 = _mm_sub_ps(mul03, mul04);
        let add01 = _mm_add_ps(sub01, mul05);
        let inv1 = _mm_mul_ps(sign_a, add01);

        let mul06 = _mm_mul_ps(vec0, fac1);
        let mul07 = _mm_mul_ps(vec1, fac3);
        let mul08 = _mm_mul_ps(vec3, fac5);
        let sub02 = _mm_sub_ps(mul06, mul07);
        let add02 = _mm_add_ps(sub02, mul08);
        let inv2 = _mm_mul_ps(sign_b, add02);

        let mul09 = _mm_mul_ps(vec0, fac2);
        let mul10 = _mm_mul_ps(vec1, fac4);
        let mul11 = _mm_mul_ps(vec2, fac5);
        let sub03 = _mm_sub_ps(mul09, mul10);
        let add03 = _mm_add_ps(sub03, mul11);
        let inv3 = _mm_mul_ps(sign_a, add03);

        let row0 = _mm_shuffle_ps(inv0, inv1, 0b00_00_00_00);
        let row1 = _mm_shuffle_ps(inv2, inv3, 0b00_00_00_00);
        let row2 = _mm_shuffle_ps(row0, row1, 0b10_00_10_00);

        let dot0 = Vec3::dot(x_axis.into(), row2.into());
        assert!(dot0 != 0.0);

        let rcp0 = _mm_set1_ps(dot0.recip());

        Mat44 {
            row: [
                _mm_mul_ps(inv0, rcp0),
                _mm_mul_ps(inv1, rcp0),
                _mm_mul_ps(inv2, rcp0),
                _mm_mul_ps(inv3, rcp0),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::{Mat44, Vec3};

    #[test]
    fn transform_point() {
        let point = Vec3::from_f32([1.0, 1.0, 1.0, 1.0]);
        let mat = Mat44::from_translation([-1.0, 5.0, -2.0]);
        let transformed = mat.transform_point(point).divided_by_w();
        assert_eq!(transformed.x(), 0.0);
        assert_eq!(transformed.y(), 6.0);
        assert_eq!(transformed.z(), -1.0);
    }
}
