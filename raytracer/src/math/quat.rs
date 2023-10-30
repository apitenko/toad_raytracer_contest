pub struct Quat {
    pub data: [f32; 4], // x y z w
}

impl Quat {
    #[inline]
    pub fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { data: [x, y, z, w] }
    }
    /// Creates a quaternion from the `angle` (in radians) around the x axis.
    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        let (s, c) = (angle * 0.5).sin_cos();
        Self::from_xyzw(s, 0.0, 0.0, c)
    }

    /// Creates a quaternion from the `angle` (in radians) around the y axis.
    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        let (s, c) = (angle * 0.5).sin_cos();
        Self::from_xyzw(0.0, s, 0.0, c)
    }

    /// Creates a quaternion from the `angle` (in radians) around the z axis.
    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        let (s, c) = (angle * 0.5).sin_cos();
        Self::from_xyzw(0.0, 0.0, s, c)
    }
}
