use crate::math::Mat44;

#[derive(Clone, Debug)]
pub struct TextureTransform {
    pub scale: [f32; 2],
    pub rotation: f32,
    pub offset: [f32; 2],
    pub matrix: Mat44,
}

impl Default for TextureTransform {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0],
            rotation: 0.0,
            offset: [0.0, 0.0],
            matrix: Mat44::IDENTITY,
        }
    }
}
