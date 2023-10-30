use std::f32::consts::PI;

use crate::math::{f32_util::Saturatable, Vec3};

// TODO: looks scuffed, read the actual paper and validate
// TODO: this is only for direct lighting, make an indirect diffuse version
// https://www.youtube.com/watch?v=CLISody-hO4
pub fn direct_oren_nayar(N: Vec3, L: Vec3, V: Vec3, NdotL: f32, NdotV: f32, roughness: f32) -> f32 {
    let theta_r = NdotV.acos();
    let theta_i = NdotL.acos();

    let cos_phi_diff =
        Vec3::dot((V - N * NdotV).normalized(), (L - N * NdotL).normalized()).saturate();

    let alpha = f32::max(theta_i, theta_r);
    let beta = f32::min(theta_i, theta_r);

    let A = 1.0 - 0.5 * roughness / (roughness + 0.33);
    let B = 0.45 * roughness / (roughness + 0.09);

    // NOTE: diffuse_reflectance / PI * <expr>
    return NdotL * (A + B * f32::max(0.0, cos_phi_diff) * alpha.sin() * beta.tan());
}

// pub fn f() {
//     let diffuse = direct_oren_nayar() * ColorTexture * ColorTint;
//     let output = (Diffuse + Ambient) * LightColor;
// }

pub fn beckmann_to_oren_nayar_roughness(beckmann_roughness: f32) -> f32 {
    return 1.0 / f32::sqrt(2.0) * f32::atan(beckmann_roughness);
}

// pub fn oren_nayar_crash(diffuse_reflectance: f32, NdotL: f32, oren_nayar_roughness: f32) -> Vec3 {

//     let cos_phiv_phil = f32::cos(phi_v - phi_l);
//     let A = 1.0 - 0.5 * om * om / (om * om + 0.33);
//     let B = 0.45 * (om * om) (om * om + 0.09);
//     let alpha = f32::max(theta_l, theta_v);
//     let beta = f32::min(theta_l, theta_v);

//     let output = diffuse_reflectance / PI * NdotL * (A + B * f32::max(0.0, cos_phiv_phil) * f32::sin(alpha) * f32::tan(beta));
//     return output;
// }
