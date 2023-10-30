use crate::math::f32_util::FloatWrapTo01;
use crate::math::{Mat44, Vec3};
use crate::scene::material::IMaterialStorage;
use image::GenericImageView;

use super::{
    samplable::Samplable,
    texture::{RawTextureData, Texture, TextureShared},
};

/// Minification filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinFilter {
    /// Corresponds to `GL_NEAREST`.
    Nearest = 1,

    /// Corresponds to `GL_LINEAR`.
    Linear,

    /// Corresponds to `GL_NEAREST_MIPMAP_NEAREST`.
    NearestMipmapNearest,

    /// Corresponds to `GL_LINEAR_MIPMAP_NEAREST`.
    LinearMipmapNearest,

    /// Corresponds to `GL_NEAREST_MIPMAP_LINEAR`.
    NearestMipmapLinear,

    /// Corresponds to `GL_LINEAR_MIPMAP_LINEAR`.
    LinearMipmapLinear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MagFilter {
    /// Corresponds to `GL_NEAREST`.
    Nearest = 1,

    /// Corresponds to `GL_LINEAR`.
    Linear,
}

#[derive(Clone, Debug)]
pub struct TextureMips {
    texture_with_mips: TextureShared,
    input_width: u32,
    input_height: u32,
    normalized_width: u32,
    normalized_height: u32,
    width_scale: f32,
    height_scale: f32,
    mips: [TextureMipBoxFloat; 16],
    max_mip: u32,
}

#[derive(Clone, Copy, Debug)]
struct TextureMipBox {
    pub start_x: u32,
    pub start_y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
struct TextureMipBoxFloat {
    pub start_x: f32,
    pub start_y: f32,
    pub scaled_width: f32,
    pub scaled_height: f32,
}

impl TextureMipBoxFloat {
    pub fn empty() -> Self {
        Self {
            scaled_width: 0.0,
            scaled_height: 0.0,
            start_x: 0.0,
            start_y: 0.0,
        }
    }
}

impl TextureMips {
    fn mip_coordinates_f32_scaled(
        normalized_width: u32,
        normalized_height: u32,
        mip_level: u32,
        width_scale: f32,
        height_scale: f32,
    ) -> TextureMipBoxFloat {
        let coordinates = Self::mip_coordinates(normalized_width, normalized_height, mip_level);

        TextureMipBoxFloat {
            scaled_height: coordinates.height as f32 * height_scale - f32::EPSILON,
            scaled_width: coordinates.width as f32 * width_scale - f32::EPSILON,
            start_x: coordinates.start_x as f32,
            start_y: coordinates.start_y as f32,
        }
    }

    fn mip_coordinates(
        normalized_width: u32,
        normalized_height: u32,
        mip_level: u32,
    ) -> TextureMipBox {
        // ! assuming that full_width == full_height and is a power of 2
        if mip_level == 0 {
            return TextureMipBox {
                start_x: 0,
                start_y: 0,
                width: normalized_width,
                height: normalized_height,
            };
        }

        let start_x = normalized_width;
        let start_y = 0 + normalized_height / u32::pow(2, mip_level);
        let width = normalized_width / u32::pow(2, mip_level);
        let height = normalized_height / u32::pow(2, mip_level);

        TextureMipBox {
            start_x,
            start_y,
            width,
            height,
        }
    }

    fn generate_mips_recursive(
        texture: &mut RawTextureData,
        full_width: u32,
        full_height: u32,
        mip_level: u32,
        filter: MinFilter,
        previous_coordinates: TextureMipBox,
    ) {
        // ! assuming that full_width == full_height and is a power of 2

        let coordinates = Self::mip_coordinates(full_width, full_height, mip_level);

        match filter {
            MinFilter::LinearMipmapLinear | MinFilter::LinearMipmapNearest => unsafe {
                for (x_index, x_target) in
                    (coordinates.start_x..(coordinates.start_x + coordinates.width)).enumerate()
                {
                    for (y_index, y_target) in (coordinates.start_y
                        ..(coordinates.start_y + coordinates.height))
                        .enumerate()
                    {
                        let p00 = texture
                            .get_pixel(
                                previous_coordinates.start_x + x_index as u32 * 2,
                                previous_coordinates.start_y + y_index as u32 * 2,
                            )
                            .0;
                        let p10 = texture
                            .get_pixel(
                                previous_coordinates.start_x + x_index as u32 * 2 + 1,
                                previous_coordinates.start_y + y_index as u32 * 2,
                            )
                            .0;
                        let p01 = texture
                            .get_pixel(
                                previous_coordinates.start_x + x_index as u32 * 2,
                                previous_coordinates.start_y + y_index as u32 * 2 + 1,
                            )
                            .0;
                        let p11 = texture
                            .get_pixel(
                                previous_coordinates.start_x + x_index as u32 * 2 + 1,
                                previous_coordinates.start_y + y_index as u32 * 2 + 1,
                            )
                            .0;
                        let current_pixel = texture.get_pixel_mut(x_target, y_target);

                        // linear interpolation
                        let interpolated_pixel_color = {
                            // todo: this calls for avx
                            let p00_ = Vec3::from_f32(p00);
                            let p01_ = Vec3::from_f32(p01);
                            let p10_ = Vec3::from_f32(p10);
                            let p11_ = Vec3::from_f32(p11);
                            ((p00_ + p01_) + (p10_ + p11_)) / 4.0
                        };
                        current_pixel.0 = interpolated_pixel_color.extract();
                    }
                }
            },
            _ => {
                for (x_index, x_target) in
                    (coordinates.start_x..(coordinates.start_x + coordinates.width)).enumerate()
                {
                    for (y_index, y_target) in (coordinates.start_y
                        ..(coordinates.start_y + coordinates.height))
                        .enumerate()
                    {
                        let p00 = texture
                            .get_pixel(
                                previous_coordinates.start_x + x_index as u32 * 2,
                                previous_coordinates.start_y + y_index as u32 * 2,
                            )
                            .0;
                        let current_pixel = texture.get_pixel_mut(x_target, y_target);
                        current_pixel.0 = p00;
                    }
                }
            }
        }

        if coordinates.width != 1 && coordinates.height != 1 {
            return Self::generate_mips_recursive(
                texture,
                full_width,
                full_height,
                mip_level + 1,
                filter,
                coordinates,
            );
        }
        // Ok(())
    }

    pub unsafe fn generate_mips(
        storage: &mut dyn IMaterialStorage,
        input_texture: &Texture,
        filter: MinFilter,
    ) -> Self {
        let input_raw = input_texture.get_raw_data();

        fn ceil_to_power_of_2(value: u32) -> u32 {
            let value = value as f32;
            let ceiled = f32::log2(value).ceil() as u32;
            u32::pow(2, ceiled)
        }
        let normalized_width = u32::max(2, ceil_to_power_of_2(input_raw.width()));
        let normalized_height = u32::max(2, ceil_to_power_of_2(input_raw.height()));

        let texture_width = normalized_width + normalized_width / 2;
        let texture_height = normalized_height;
        let mut texture_with_mips = RawTextureData::new(texture_width, texture_height);

        // Copy the base texture
        for (input_x, input_y, input_pixel) in input_raw.enumerate_pixels() {
            let pixel = texture_with_mips.get_pixel_mut(input_x, input_y);
            pixel.0 = input_pixel.0;
        }
        // Recursive process
        Self::generate_mips_recursive(
            &mut texture_with_mips,
            normalized_width,
            normalized_height,
            1,
            filter,
            Self::mip_coordinates(normalized_width, normalized_height, 0),
        );

        let input_width = input_raw.width();
        let input_height = input_raw.height();
        let width_scale = input_width as f32 / normalized_width as f32 - f32::EPSILON;
        let height_scale = input_height as f32 / normalized_height as f32 - f32::EPSILON;

        let texture = Texture::new(
            texture_with_mips,
            texture_width as usize,
            texture_height as usize,
        );
        let texture_with_mips = storage.push_texture(texture);

        let max_mip = u32::min(16u32, u32::ilog2(normalized_width));
        let mips = {
            let mut mips = [TextureMipBoxFloat::empty(); 16];
            for i in 0..max_mip {
                mips[i as usize] = Self::mip_coordinates_f32_scaled(
                    normalized_width,
                    normalized_height,
                    i,
                    width_scale,
                    height_scale,
                );
            }
            mips
        };
        Self {
            texture_with_mips,
            input_width,
            input_height,
            normalized_width,
            normalized_height,
            width_scale,
            height_scale,
            mips,
            max_mip,
        }
    }

    #[inline]
    pub fn sample(&self, u: f32, v: f32, mip: usize, texture_transform: &TextureTransform) -> Vec3 {
        let coordinates = &self.mips[mip];

        let v = Vec3::from_f32([u, v, 0.0, 0.0]);
        let transformed_v = texture_transform.matrix * v;
        let u = transformed_v.x();
        let v = transformed_v.y();

        let x: usize = (coordinates.start_x + u.wrap_01() * coordinates.scaled_width) as usize;
        let y: usize = (coordinates.start_y + v.wrap_01() * coordinates.scaled_height) as usize;
        let sample = unsafe {
            self.texture_with_mips
                .get()
                .get_raw_data()
                .unsafe_get_pixel(x as u32, y as u32)
        };
        return Vec3::from_f32(sample.0).as_vector();
    }
}

#[derive(Clone, Debug)]
pub struct Sampler {
    texture_mips: TextureMips,
    min_filter: MinFilter,
    mag_filter: MagFilter,
    tex_coord_index: usize,
    texture_transform: TextureTransform,
}

impl Sampler {
    pub fn new(
        storage: &mut dyn IMaterialStorage,
        texture: Texture,
        min_filter: MinFilter,
        mag_filter: MagFilter,
        tex_coord_index: usize,
        texture_transform: TextureTransform,
    ) -> Self {
        unsafe {
            let texture_mips = TextureMips::generate_mips(storage, &texture, min_filter);
            Self {
                texture_mips,
                min_filter,
                mag_filter,
                tex_coord_index,
                texture_transform,
            }
        }
    }
}

impl Samplable for Sampler {
    fn sample(&self, uv: &[(f32, f32); 4], mip: f32) -> Vec3 {
        // TODO: cross-layer sampling (bilinear/aniso)
        // let mip: f32 = 0.0;
        let mip = f32::clamp(mip, 0.0, (self.texture_mips.max_mip - 1) as f32);
        self.texture_mips.sample(
            uv[self.tex_coord_index].0,
            uv[self.tex_coord_index].1,
            mip.floor() as usize,
            &self.texture_transform
        )
    }
}

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
