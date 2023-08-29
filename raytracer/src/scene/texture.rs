use std::{path::Path, ptr::null};

use base64::Engine;
use image::GenericImageView;

use crate::math::Vec3;

#[derive(Clone)]
pub struct Texture {
    width: usize,
    height: usize,
    image: RawTextureData,
}

impl Texture {
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let x: usize = (u.fract() * 0.99 * self.width as f32) as usize;
        let y: usize = (v.fract() * 0.99 * self.height as f32) as usize;
        let sample = unsafe { self.image.unsafe_get_pixel(x as u32, y as u32) };
        return Vec3::from_f32(sample.0);
    }
}

#[derive(Clone, Copy)]
pub struct TextureShared {
    mat: *const Texture,
}

unsafe impl Send for TextureShared {}
unsafe impl Sync for TextureShared {}

impl TextureShared {
    pub fn uninitialized() -> Self {
        Self {
            mat: null()
        }
    }
    pub const fn new(mat: *const Texture) -> Self {
        Self { mat }
    }

    pub fn get(&self) -> &Texture {
        unsafe {
            return &*self.mat as &Texture;
        }
    }
}

// Texture loader

pub type RawTextureData = image::ImageBuffer<image::Rgba<f32>, Vec<f32>>;

impl Texture {
    pub fn new_from_image(image: RawTextureData) -> anyhow::Result<Self> {
        let height = image.height();
        let width = image.width();
        Ok(Self {
            width: width as usize, // wtf rust? casting should be unnecessary
            height: height as usize,
            image,
        })
    }

    pub fn new_from_raw_bytes(data: &[u8]) -> anyhow::Result<Self> {
        let img = image::io::Reader::new(std::io::Cursor::new(data))
            .with_guessed_format()?
            .decode()?;

        let img_data = img.to_rgba32f();

        return Self::new_from_image(img_data);
    }

    pub fn new_from_file(filepath: &Path) -> anyhow::Result<Self> {
        // let filepath = "./resources/uuu.jpg";
        let texture_file = std::fs::read(filepath)?;
        return Self::new_from_raw_bytes(&texture_file);
    }

    pub fn new_from_base64(base64: &[u8]) -> anyhow::Result<Texture> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(base64)
            .unwrap();

        let texture = Texture::new_from_raw_bytes(&bytes)?;
        return Ok(texture);
    }

    pub fn new_from_base64_str(base64_str: &str) -> anyhow::Result<Self> {
        
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(base64_str)
            .unwrap();
        let texture = Texture::new_from_raw_bytes(&bytes)?;
        return Ok(texture);
    }

    pub fn make_default_texture() -> anyhow::Result<Texture> {
        // https://shoonia.github.io/1x1/#ffffffff
        const WHITE_PIXEL_PNG_BASE64: &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAAXNSR0IArs4c6QAAAA1JREFUGFdj+P///38ACfsD/QVDRcoAAAAASUVORK5CYII=";
        // const MAGENTA_PIXEL_PNG_BASE64: &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAAEnQAABJ0Ad5mH3gAAAAMSURBVBhXY/jP8B8ABAAB/4jQ/cwAAAAASUVORK5CYII=";

        return Self::new_from_base64(WHITE_PIXEL_PNG_BASE64);
    }
}
