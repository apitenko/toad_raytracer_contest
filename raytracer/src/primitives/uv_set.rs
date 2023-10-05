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

#[derive(Clone, Debug)]
pub struct UVChannel {
    pub points: [[f32;2]; 3],
}

#[derive(Clone, Debug)]
pub struct UVSet {
    pub channels: [UVChannel; 4],
}

impl UVSet {
    #[inline]
    pub const fn empty() -> Self {
        UVSet {
            channels: [
                UVChannel {
                    points: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
                },
                UVChannel {
                    points: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
                },
                UVChannel {
                    points: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
                },
                UVChannel {
                    points: [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
                },
            ],
        }
    }

    #[inline]
    pub fn read(input_uv: &[Vec<[f32; 2]>; 4], i0: usize, i1: usize, i2: usize) -> Self {
        let get_points = |channel_index: usize| {
            let channel = &input_uv[channel_index];
            let uv0 = channel[i0 as usize];
            let uv1 = channel[i1 as usize];
            let uv2 = channel[i2 as usize];
            return UVChannel {
                points: [uv0.into(), uv1.into(), uv2.into()],
            };
        };

        UVSet {
            channels: [get_points(0), get_points(1), get_points(2), get_points(3)],
        }
    }
}
