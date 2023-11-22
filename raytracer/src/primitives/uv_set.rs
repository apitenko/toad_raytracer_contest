// #[derive(Clone, Copy, Debug)]
// pub struct UV {
//     u: f32,
//     v: f32,
// }

// impl From<[f32; 2]> for UV {
//     fn from(value: [f32; 2]) -> Self {
//         UV {
//             u: value[0],
//             v: value[1],
//         }
//     }
// }

use std::mem::MaybeUninit;

use crate::{scene::{material::Material, texture::texture_transform::TextureTransform}, math::Vec3};

#[derive(Clone, Debug)]
pub struct UVChannel {
    pub points: [[f32; 2]; 3],
}

#[derive(Clone, Debug)]
pub struct UVSet {
    pub channels_color: [UVChannel; 4],
    pub channels_metalrough: [UVChannel; 4],
    pub channels_normalmap: [UVChannel; 4],
    pub channels_emission: [UVChannel; 4],
    pub channels_transmission: [UVChannel; 4],
}

impl UVSet {
    #[inline]
    pub const fn empty() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }

    #[inline]
    pub fn read(
        input_uv: &[Vec<[f32; 2]>; 4],
        i0: usize,
        i1: usize,
        i2: usize,
        material: &Material,
    ) -> Self {
        let texture_transform_color = &material.color_texture.texture_transform;
        let texture_transform_metalrough = &material.metallic_roughness_texture.texture_transform;
        let texture_transform_normalmap = &material.normal_texture.texture_transform;
        let texture_transform_emission = &material.emission_texture.texture_transform;
        let texture_transform_transmission = &material.transmission_texture.texture_transform;

        let fn_transform = |texture_transform: &TextureTransform, uv: [f32; 2]| {
            let v = Vec3::from_f32([uv[0], uv[1], 0.0, 0.0]);
            let transformed_v = texture_transform.matrix * v;
            let u = transformed_v.x();
            let v = transformed_v.y();
            return [u, v];
        };

        let get_points = |texture_transform: &TextureTransform, channel_index: usize| {
            let channel = &input_uv[channel_index];
            let uv0 = fn_transform(texture_transform, channel[i0 as usize]);
            let uv1 = fn_transform(texture_transform, channel[i1 as usize]);
            let uv2 = fn_transform(texture_transform, channel[i2 as usize]);
            return UVChannel {
                points: [uv0.into(), uv1.into(), uv2.into()],
            };
        };

        UVSet {
            channels_color: [
                get_points(texture_transform_color, 0),
                get_points(texture_transform_color, 1),
                get_points(texture_transform_color, 2),
                get_points(texture_transform_color, 3),
            ],
            channels_metalrough: [
                get_points(texture_transform_metalrough, 0),
                get_points(texture_transform_metalrough, 1),
                get_points(texture_transform_metalrough, 2),
                get_points(texture_transform_metalrough, 3),
            ],
            channels_normalmap: [
                get_points(texture_transform_normalmap, 0),
                get_points(texture_transform_normalmap, 1),
                get_points(texture_transform_normalmap, 2),
                get_points(texture_transform_normalmap, 3),
            ],
            channels_emission: [
                get_points(texture_transform_emission, 0),
                get_points(texture_transform_emission, 1),
                get_points(texture_transform_emission, 2),
                get_points(texture_transform_emission, 3),
            ],
            channels_transmission: [
                get_points(texture_transform_transmission, 0),
                get_points(texture_transform_transmission, 1),
                get_points(texture_transform_transmission, 2),
                get_points(texture_transform_transmission, 3),
            ]
        }
    }
}
