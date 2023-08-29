use crate::{math::Vec3, surface::TotallySafeSurfaceWrapper};

// for testing if sRGB is enabled
pub fn fill_gradient_black_to_white(
    mut surface: TotallySafeSurfaceWrapper
) {
    let uv_start = (0.2, 0.2);
    let uv_end = (0.8, 0.8);

    let xy_start = (
        (uv_start.0 * surface.width() as f32) as u32,
        (uv_start.1 * surface.height() as f32) as u32,
    );
    let xy_end = (
        (uv_end.0 * surface.width() as f32) as u32,
        (uv_end.1 * surface.height() as f32) as u32,
    );

    let width = xy_end.0 - xy_start.0;

    let color_start = Vec3::ZERO;
    let color_end = Vec3::ONE;

    for x in 0..surface.width() {
        for y in 0..surface.height() {
            surface.write((x, y), Vec3::ONE / 2.0);
        }
    }

    for x in xy_start.0..xy_end.0 {
        for y in xy_start.1..xy_end.1 {
            let t = (x - xy_start.0) as f32 / width as f32;
            let color = Vec3::lerp(color_start, color_end, t);
            surface.write((x, y), color);
        }
    }
}
