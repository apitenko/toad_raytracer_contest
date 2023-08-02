use lazy_static::lazy_static;

use crate::math::Vec3;

pub struct Texture {
    width: usize,
    height: usize,
    data: Vec<u32>, // RBGA or something idk
}

impl Texture {
    pub const fn new(width: usize, height: usize, data: Vec<u32>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let x: usize = (u * self.width as f32) as usize;
        let y: usize = (v * self.height as f32) as usize;
        let index = (y * self.height + x);
        let index = index.clamp(0, self.width * self.height - 1);
        let packed = self.data[index as usize];
        Vec3::from_packed_u32_rgb(packed)
    }
}

#[derive(Clone)]
pub struct TextureShared {
    mat: *const Texture,
}

unsafe impl Send for TextureShared {}
unsafe impl Sync for TextureShared {}

impl TextureShared {
    pub const fn new(mat: *const Texture) -> Self {
        Self { mat }
    }

    pub fn get(&self) -> &Texture {
        unsafe {
            return &*self.mat as &Texture;
        }
    }

    pub fn make_default_texture() -> Self {
        let default_texture = Box::new(Texture::new(1, 1, vec![0xFFFFFFFFu32]));
        let default_texture_ref = Box::leak(default_texture);
        TextureShared::new(default_texture_ref as *const Texture)
    }
}

pub fn make_checkerboard_texture() -> Texture {
    let width = 400;
    let height = 200;
    let mut data = vec![0; width * height];

    for i in 0..height {
        for j in 0..width {
            if (i + j) % 2 == 0 {
                data[i * width + j] = 0xCCCCCCCC;
            } else {
                data[i * width + j] = 0x11111111;
            }
        }
    }

    return Texture::new(width, height, data);
}
