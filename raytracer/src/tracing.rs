use std::f32::consts::PI;

use rand::Rng;

use crate::constants::{
    AMBIENT_LIGHT_COLOR, AMBIENT_LIGHT_INTENSITY, COLOR_BLUE, FILTER_GLOSSY, FLOAT_ERROR,
    MAX_BOUNCES, SKYBOX_COLOR, SKYBOX_LIGHT_INTENSITY,
};
use crate::scene::acceleration_structure::acceleration_structure::AccelerationStructure;
use crate::scene::lights::light::attenuation_fn;
use crate::util::prng::{rand01, rand_range};
use crate::{
    constants::{COLOR_RED, COLOR_SKY_BLUE, COLOR_WHITE, MISS_COLOR_VEC3},
    math::{
        f32_util::Saturatable,
        ray::{reflect, RayRefractionState},
        Ray, RayBounce, Vec3,
    },
    primitives::cast_result::CastResult,
    scene::{lights::light::Light, material::Material, scene::Scene},
    util::fresnel_constants::FresnelConstants,
};

// Cook-Torrance F term
fn schlick_fresnel(f0: Vec3, LdotH: f32) -> Vec3 {
    return f0 + (Vec3::ONE - f0) * f32::powi(1.0 - LdotH, 5);
}

fn schlick_fresnel_f0(n1: f32, n2: f32) -> f32 {
    let mut r0: f32 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    return r0;
}

// const OBJECT_REFLECTIVITY: f32 = 0.01;
fn fresnel_reflect_amount(n1: f32, n2: f32, LdotH: f32) -> f32 {
    // #if DO_FRESNEL
    // Schlick aproximation
    let mut r0: f32 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    let mut cosX: f32 = LdotH;
    if n1 > n2 {
        let n: f32 = n1 / n2;
        let sinT2: f32 = n * n * (1.0 - cosX * cosX);
        // Total internal reflection
        if sinT2 > 1.0 {
            return 1.0;
        }
        cosX = (1.0 - sinT2).sqrt();
    }
    let x: f32 = 1.0 - cosX;
    let ret: f32 = r0 + (1.0 - r0) * x * x * x * x * x;

    // adjust reflect multiplier for object reflectivity
    // ret = (OBJECT_REFLECTIVITY + (1.0 - OBJECT_REFLECTIVITY) * ret);
    return ret;
    // #else
    // 	return OBJECT_REFLECTIVITY;
    // #endif
}

// fn FresnelSchlick(F0: f32, cosTheta: f32) -> f32 {
//     return F0 + (1.0 - F0) * pow(1.0 - saturate(cosTheta), 5.0);
// }

pub fn ray_cast(current_bounce: RayBounce, scene: &Scene) -> Vec3 {
    if current_bounce.current_bounces > MAX_BOUNCES {
        // stop recursion by limit
        return Vec3::ZERO;
    }
    // if current_bounce.remaining_depth < 0.00001 {
    //     return Vec3::ZERO;
    // }

    let cast_result = scene
        .geometry
        .single_cast(
            current_bounce.ray,
            current_bounce.refraction_state == RayRefractionState::TraversingAir,
        )
        .resolve();

    let cast_result = if let Some(cast_result) = cast_result {
        cast_result
    } else {
        // every miss is a skybox hit
        // miss after bounce
        return SKYBOX_COLOR * SKYBOX_LIGHT_INTENSITY;
        // let unit_direction = current_bounce.ray.direction().normalized();
        // let skybox_color = scene.skybox.sample_from_direction(unit_direction);
        // return skybox_color * current_bounce.multiplier;
    };

    // let mip: f32 = current_bounce.distance / 2.0;
    let mip: f32 = 0.0;
    let current_material = cast_result.material.get();

    let material_emission = current_material.sample_emission(&cast_result.uv_emission, mip);
    if material_emission.luminosity() > 0.001 {
        return emission_brdf(material_emission);
    }

    let material_color = current_material.sample_albedo(&cast_result.uv_color, mip);

    let (material_roughness, material_metallic) =
        current_material.sample_roughness_metallic(&cast_result.uv_metalrough, mip);

    // if current_bounce.apply_filter_glossy {
    //     material_roughness = (material_roughness + FILTER_GLOSSY * current_bounce.current_bounces as f32).clamp(0.0, 1.0);
    // }
    // let material_roughness = material_roughness * material_roughness;
    let surface_normal = 'surface_normal: {
        let material_normal = current_material.sample_normal(&cast_result.uv_normalmap, mip);
        // break 'surface_normal material_normal;
        let material_normal = (2.0 * material_normal - Vec3::ONE); //.normalized();
        let surface_normal = (material_normal.z() * cast_result.normal
            + material_normal.x() * cast_result.tangent
            + material_normal.y() * cast_result.bitangent)
            .normalized();
        surface_normal
    };
    // let surface_normal = cast_result.normal;

    let material_ior = current_material.ior;

    // GGX
    const DO_DIRECT_LIGHTING: bool = true;
    const DO_INDIRECT_LIGHTING: bool = false;
    // let DO_DIRECT_LIGHTING: bool = current_bounce.current_bounces > 0;

    // Do explicit direct lighting to a random light in the scene
    let component_direct = if DO_DIRECT_LIGHTING {
        ggx_direct(
            scene,
            &cast_result,
            surface_normal,
            current_bounce.ray.direction(),
            material_color,
            material_metallic,
            material_roughness,
            material_ior,
            &current_bounce,
        )
    } else {
        Vec3::ZERO
    };

    let component_indirect = if DO_INDIRECT_LIGHTING {
        // Do indirect lighting for global illumination
        ggx_indirect(
            scene,
            &cast_result,
            surface_normal,
            &current_bounce,
            material_color,
            material_metallic,
            material_roughness,
            material_ior,
        )
    } else {
        Vec3::ZERO
    };

    // TODO: Refraction
    // Split energy between Diffuse and Refracted
    // let diffuse_multiplier = 0.5;
    // let refracted_multiplier = 1.0 - diffuse_multiplier;

    // let refraction_new_state =
    //     if let RayRefractionState::TraversingAir = current_bounce.refraction_state {
    //         RayRefractionState::InsideMaterial {
    //             current_outside_fresnel_coefficient: fresnel_outside,
    //         }
    //     } else {
    //         RayRefractionState::TraversingAir
    //     };

    // let component_refract = outside_cast(
    //     // TODO: should be inside cast
    //     RayBounce {
    //         ray: Ray::new(
    //             cast_result.intersection_point,
    //             refracted_ray_direction,
    //             f32::MAX,
    //         ),
    //         bounces: current_bounce.bounces - 1,
    //         multiplier: refracted_multiplier,
    //         refraction_state: refraction_new_state,
    //     },
    //     scene,
    // );

    // TODO: Subsurface Scattering

    // ! Blend components  -------------------------

    let final_color = component_direct
        + component_indirect
        + AMBIENT_LIGHT_INTENSITY * AMBIENT_LIGHT_COLOR * material_color;
    return final_color;
}

#[inline]
fn emission_brdf(material_emission: Vec3) -> Vec3 {
    return material_emission;
}

// Cook-Torrance D term
#[inline]
fn ggx_normal_distribution(NdotH: f32, roughness: f32) -> f32 {
    let a2 = roughness * roughness;
    let d = ((NdotH * a2 - NdotH) * NdotH + 1.0);
    return a2 / (d * d * PI);
}

// Cook-Torrance G term
// TODO: maybe find a better model
#[inline]
fn ggx_schlick_masking_term(NdotL: f32, NdotV: f32, roughness: f32) -> f32 {
    // Karis notes they use alpha / 2 (or roughness^2 / 2)
    // let k = roughness * roughness / 2.0;

    // https://learnopengl.com/PBR/Lighting
    // ^ uses slightly different roughness mapping:
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    // Compute G(v) and G(l).  These equations directly from Schlick 1994
    //     (Though note, Schlick's notation is cryptic and confusing.)
    let g_v = NdotV / (NdotV * (1.0 - k) + k);
    let g_l = NdotL / (NdotL * (1.0 - k) + k);
    return g_v * g_l;
}

// When using this function to sample, the probability density is:
//      pdf = D * NdotH / (4 * HdotV)
fn getGGXMicrofacet(roughness: f32, surface_normal: Vec3, tangent: Vec3, bitangent: Vec3) -> Vec3 {
    // Get our uniform random numbers
    // #[thread_local]
    // static mut randVal: (f32, f32) = (0.5, 0.5);
    unsafe {
        // randVal = (rand01(), rand01());
        let randVal: (f32, f32) = (rand01(), rand01());

        // GGX NDF sampling
        let a2 = roughness * roughness;
        let cosThetaH = f32::sqrt(f32::max(
            0.0,
            (1.0 - randVal.0) / ((a2 - 1.0) * randVal.0 + 1.0),
        ));
        let sinThetaH = f32::sqrt(f32::max(0.0, 1.0 - cosThetaH * cosThetaH));
        let phiH = randVal.1 * PI * 2.0;

        // Get our GGX NDF sample (i.e., the half vector)
        let output = tangent * (sinThetaH * f32::cos(phiH))
            + bitangent * (sinThetaH * f32::sin(phiH))
            + surface_normal * cosThetaH;

        // let fuck = format!("output {:?} surface_normal {:?}", output, surface_normal);
        // assert!(Vec3::dot(output, surface_normal) >= 0.0, "{}", fuck);
        // let output = Vec3::lerp(tangent, surface_normal, 0.7);
        // let output = Vec3::lerp(output, surface_normal, 0.7);
        return output;
    }
}

fn ggx_direct(
    scene: &Scene,
    cast_result: &CastResult,
    surface_normal: Vec3,
    current_ray_direction: Vec3,
    material_color: Vec3,
    material_metallic: f32,
    material_roughness: f32,
    material_ior: f32,
    current_bounce: &RayBounce,
) -> Vec3 {
    let V = -current_ray_direction;
    let N = surface_normal;

    //////
    let fn_sample_light = |light_source: &dyn Light| {
        let (distance_to_light, normal_into_light) =
            light_source.normal_from(cast_result.intersection_point);

        let L = normal_into_light;
        // Compute our lambertian term (N dot L)
        let NdotL = Vec3::dot(surface_normal, L).saturate();

        let light_intensity = light_source.get_emission(cast_result.intersection_point);
        let light_visibility = shadow_ray_visibility(light_source, scene, cast_result);

        // return light_intensity * light_visibility * NdotL * NdotL; // simple model for testing

        // Compute half vectors and additional dot products for GGX
        let H: Vec3 = (V + L).normalized();
        let NdotH = (Vec3::dot(N, H)).saturate();
        let LdotH = (Vec3::dot(L, H)).saturate();
        let NdotV = (Vec3::dot(N, V)).saturate();
        let HdotV = (Vec3::dot(H, V)).saturate(); // same as LdotH?

        // Evaluate terms for our GGX BRDF model
        let D = ggx_normal_distribution(NdotH, material_roughness);
        let G = ggx_schlick_masking_term(NdotL, NdotV, material_roughness);

        let dielectric_f0 = schlick_fresnel_f0(FresnelConstants::Air, material_ior);
        let dielectric_f0 = Vec3::new([dielectric_f0, dielectric_f0, dielectric_f0]);
        let f0 = Vec3::lerp(dielectric_f0, material_color, material_metallic); // color channel as albedo for metallics
        let F: Vec3 = schlick_fresnel(f0, HdotV);

        // Evaluate the Cook-Torrance Microfacet BRDF model
        //     Cancel NdotL here to avoid catastrophic numerical precision issues.
        let ggx_specular: Vec3 = /* NdotL * */ Vec3::ONE * D * G * F / (4.0 * NdotV/* * NdotL */);
        // let ggx_specular = Vec3::ZERO;

        let kS = F;
        let ratio_or_refraction = (Vec3::ONE - kS) * (1.0 - material_metallic);

        let lambertian_diffuse = NdotL * ratio_or_refraction * material_color / PI;

        // Compute our final color (combining diffuse lobe plus specular GGX lobe)
        return light_visibility * light_intensity * (ggx_specular + lambertian_diffuse);
    };

    if current_bounce.monte_carlo_reached() {
        let random_light = {
            // Pick a random light from our scene to shoot a shadow ray towards
            let lights_count = scene.lights.len();
            let random_light_index = rand_range(lights_count);
            let random_light = scene.lights[random_light_index].as_ref();
            random_light
        };
        return fn_sample_light(random_light);
    } else {
        let mut color = Vec3::ZERO;
        for light in &scene.lights {
            color += fn_sample_light(light.as_ref());
        }
        return color / scene.lights.len() as f32;
    }
}

fn shadow_ray_visibility(
    light_source: &dyn Light,
    scene: &Scene,
    cast_result: &CastResult,
) -> Vec3 {
    let (distance_to_light, normal_into_light) =
        light_source.normal_from(cast_result.intersection_point);

    let light_cast_result = scene.geometry.single_cast(
        Ray::new(
            cast_result.intersection_point,// + 0.01 * cast_result.normal,
            normal_into_light,
            distance_to_light,
        ),
        false,
    );

    if !light_cast_result.has_missed() {
        return Vec3::ZERO;
    } else {
        return Vec3::ONE;
    }
}

fn ggx_indirect(
    scene: &Scene,
    cast_result: &CastResult,
    surface_normal: Vec3,
    current_bounce: &RayBounce,
    material_color: Vec3,
    material_metallic: f32,
    material_roughness: f32,
    material_ior: f32,
) -> Vec3 {
    // ugh
    let current_ray_direction = current_bounce.ray.direction();
    let V = -current_bounce.ray.direction();
    let N = surface_normal;
    let hit = cast_result.intersection_point;
    let tangent = cast_result.tangent;
    let bitangent = cast_result.bitangent;

    // let material_roughness: f32 = 0.0;
    // if cast_result.material.get().emission_factor.x() == 0.0 {
    //     println!("Hitto");
    // }
    // let material_roughness = 0.95;

    // ! Light model for geometry tests ////////////////////////////////
    // const SIMPLE_INDIRECT: bool = false;
    // if SIMPLE_INDIRECT {
    //     let reflected = reflect(current_ray_direction, N);
    //     let bounce_color: Vec3 = ray_cast(
    //         RayBounce {
    //             ray: Ray::new(
    //                 hit + FLOAT_ERROR * reflected,
    //                 reflected,
    //                 f32::MAX,
    //             ),
    //             current_bounces: current_bounce.current_bounces + 1,
    //             distance: current_bounce.distance + cast_result.distance_traversed,
    //             refraction_state: RayRefractionState::TraversingAir,
    //             // apply_filter_glossy: true
    //         },
    //         scene,
    //     );
    //     return (bounce_color + Vec3::ONE) / 6.0 * (Vec3::lerp(material_color, Vec3::ONE, 0.1));
    // }
    // ! //////////////////////////////////////////////////////////////

    let fn_diffuse_ray = |probDiffuse: f32| {
        // return Vec3::ZERO;
        // Shoot a randomly selected cosine-sampled diffuse ray.
        let random_direction: Vec3 = (get_cos_hemisphere_sample(N, tangent, bitangent));
        let mut bounce_color: Vec3 = ray_cast(
            RayBounce {
                ray: Ray::new(
                    hit + FLOAT_ERROR * random_direction,
                    random_direction,
                    f32::MAX,
                ),
                current_bounces: current_bounce.current_bounces + 1,
                distance: current_bounce.distance + cast_result.distance_traversed,
                refraction_state: RayRefractionState::TraversingAir,
                // apply_filter_glossy: true
            },
            scene,
        );

        // let bounce_lumi = bounce_color.luminosity();
        // if bounce_lumi > 3.0 {
        //     // bounce_color = bounce_color / bounce_lumi * 3.0;
        //     bounce_color = Vec3::ZERO;
        // }
        // bounce_color = Vec3::ZERO;

        // Accumulate the color: (NdotL * incomingLight * material_albedo / pi)
        // Probability of sampling this ray:  (NdotL / pi) * probDiffuse
        let result_color = bounce_color * material_color / probDiffuse;
        return result_color;
    };
    let fn_specular_ray = |probDiffuse: f32| {
        // return Vec3::ZERO;
        // Randomly sample the NDF to get a microfacet in our BRDF
        let H: Vec3 = getGGXMicrofacet(material_roughness, N, tangent, bitangent).normalized();

        // Compute outgoing direction based on this (perfectly reflective) facet
        let reflected_ray = reflect(current_ray_direction, H);

        // Compute our color by tracing a ray in this direction
        let bounce_color: Vec3 = ray_cast(
            RayBounce {
                ray: Ray::new(hit + FLOAT_ERROR * reflected_ray, reflected_ray, f32::MAX),
                current_bounces: current_bounce.current_bounces + 1,
                distance: current_bounce.distance + cast_result.distance_traversed,
                refraction_state: RayRefractionState::TraversingAir,
                // apply_filter_glossy: false,
            },
            scene,
        );
        let L = reflected_ray;

        // Compute some dot products needed for shading
        let NdotL: f32 = (Vec3::dot(N, L)).saturate();
        let NdotH: f32 = (Vec3::dot(N, H)).saturate();
        let LdotH: f32 = (Vec3::dot(L, H)).saturate();
        let NdotV: f32 = (Vec3::dot(N, V)).saturate();

        // Evaluate our BRDF using a microfacet BRDF model
        // let D: f32 = ggx_normal_distribution(NdotH, material_roughness);
        let G: f32 = ggx_schlick_masking_term(NdotL, NdotV, material_roughness);
        // let F: Vec3 = schlick_fresnel(Vec3::ONE / 2.0, LdotH);
        let F: f32 = fresnel_reflect_amount(1.0, material_ior, LdotH);
        // let ggxTerm: f32 = D * G * F / (4.0 * NdotL * NdotV);

        // return Vec3::ONE * H;
        // What's the probability of sampling vector H from getGGXMicrofacet()?
        // let ggxProb: f32 = (D * NdotH / (4.0 * LdotH)).saturate();
        // return ggxProb * Vec3::ONE;

        // Accumulate color:  ggx-BRDF * lightIn * NdotL / probability-of-sampling
        //    -> Note: Should really cancel and simplify the math above
        // return NdotL * bounce_color * ggxTerm / (ggxProb * (1.0 - diffuseMult));
        // return Vec3::ONE * ggxProb;

        // NOTE: this is simplified version of a line earlier
        return bounce_color * G * F * LdotH / (NdotH * NdotV * (1.0 - probDiffuse));
    };

    let fn_specular_metallic_ray = || {
        // return Vec3::ZERO;
        // Randomly sample the NDF to get a microfacet in our BRDF
        // println!("{:?}", material_roughness);
        let H: Vec3 = getGGXMicrofacet(material_roughness, N, tangent, bitangent).normalized();

        // Compute outgoing direction based on this (perfectly reflective) facet
        let reflected_ray = reflect(current_ray_direction, H);

        // let NdotH: f32 = (Vec3::dot(N, H)).saturate();
        // let HdotV: f32 = (Vec3::dot(H, V)).saturate();

        // let D: f32 = ggx_normal_distribution(NdotH, material_roughness);
        // let ggxProb: f32 = (D * NdotH / (4.0 * HdotV)).saturate();

        // Compute our color by tracing a ray in this direction
        let bounce_color: Vec3 = ray_cast(
            RayBounce {
                ray: Ray::new(hit + FLOAT_ERROR * reflected_ray, reflected_ray, f32::MAX),
                current_bounces: current_bounce.current_bounces + 1,
                distance: current_bounce.distance + cast_result.distance_traversed,
                refraction_state: RayRefractionState::TraversingAir,
                // apply_filter_glossy: false,
            },
            scene,
        );

        // if bounce_color.luminosity() > 10.0 {
        //     println!("Yeet {:?}", bounce_color);
        //     let bounce_color: Vec3 = ray_cast(
        //         RayBounce {
        //             ray: Ray::new(hit + FLOAT_ERROR * reflected_ray, reflected_ray, f32::MAX),
        //             current_bounces: current_bounce.current_bounces + 1,
        //             distance: current_bounce.distance + cast_result.distance_traversed,
        //             refraction_state: RayRefractionState::TraversingAir,
        //         },
        //         scene,
        //     );
        //     // println!("Yeet {:?}", bounce_color);
        // }

        // color is albedo
        return bounce_color * material_color;
    };

    // if rand01() < material_metallic && material_metallic > 0.01 {
    //     // ! metallic
    //     return fn_specular_metallic_ray();
    // }
    // return Vec3::ZERO;
    // return fn_specular_metallic_ray();
    // let output = {
    //     let calculated_specular = schlick_fresnel_ior_to_specular(1.0, material_ior);
    //     let diffuse = Vec3::ZERO;
    //     let diffuse = fn_diffuse_ray(0.06);
    //     let specular = Vec3::ZERO;
    //     // let specular = fn_specular_ray(1.0 - calculated_specular);
    //     let metallic = Vec3::ZERO;
    //     // let metallic = fn_specular_metallic_ray();
    //     (diffuse + specular) / 2.0 * (1.0 - material_metallic) + metallic * material_metallic
    // };

    // if output.luminosity() > 3.0 {
    //     println!("Output {:?}", output);
    //     let output = {
    //         let calculated_specular = schlick_fresnel_ior_to_specular(1.0, material_ior);
    //         let diffuse = fn_diffuse_ray(calculated_specular);
    //         let specular = fn_specular_ray(calculated_specular);
    //         let metallic = fn_specular_metallic_ray();
    //         (diffuse + specular) / 2.0 * (1.0 - material_metallic) + metallic * material_metallic
    //     };
    // }
    // return output;
    // return fn_specular_metallic_ray();
    return fn_diffuse_ray(0.5);

    if rand01() < material_metallic && material_metallic > 0.001 {
        // ! metallic
        // return Vec3::ZERO;
        return fn_specular_metallic_ray();
    } else {
        // ! dielectric
        let calculated_specular = schlick_fresnel_f0(FresnelConstants::Air, material_ior);
        // color is diffuse
        let (probDiffuse, diffuseMult) =
            probability_to_sample_diffuse(material_color, calculated_specular);

        let chooseDiffuse = rand01() < probDiffuse;
        if chooseDiffuse {
            return fn_diffuse_ray(diffuseMult);
        } else {
            return fn_specular_ray(diffuseMult);
        }
    }
}

// TODO: find a better approach
fn probability_to_sample_diffuse(material_albedo: Vec3, material_specular: f32) -> (f32, f32) {
    let lumSpecular = material_specular;
    if lumSpecular > 0.0001 {
        let lumDiffuse = material_albedo.luminosity().saturate();
        let mult = lumDiffuse / (lumDiffuse + lumSpecular);
        return (mult, mult);
    } else {
        return (1.0, 0.5);
    }
}

// Get a cosine-weighted random vector centered around a specified normal direction.
fn get_cos_hemisphere_sample(hitNorm: Vec3, tangent: Vec3, bitangent: Vec3) -> Vec3 {
    // Get 2 random numbers to select our sample with

    // #[thread_local]
    // static mut u: f32 = 1.0;
    // #[thread_local]
    // static mut v: f32 = 1.0;

    unsafe {
        // u = rand01();
        // v = rand01();

        // let u = rand01().clamp(0.05, 0.95);
        // let v = rand01().clamp(0.05, 0.95);

        let u = rand01();
        let v = rand01();

        let r = u.sqrt();
        let phi = 2.0 * PI * v;
        let result =
            tangent * (r * phi.cos()) + bitangent * (r * phi.sin()) + hitNorm * (1.0 - v).sqrt();

        // Get our cosine-weighted hemisphere lobe sample direction
        return result.normalized();
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::math::Vec3;

//     use super::getCosHemisphereSample;

//     #[test]

//     fn testFuck() {
//         for _ in 0..100_000 {
//             let vec1 = getCosHemisphereSample(Vec3::from_f32([0.0, 0.0, -0.999969900, 0.0]));
//             let vec = vec1.normalized();
//             // if Vec3::dot(vec, Vec3::DOWN) > 0.001 {
//             //     println!("FGSDF");
//             // }
//             if vec.x() == 0.0 && vec.y() == 0.0 && vec.z() > 0.99 {
//                 println!("FGSDF");
//             }
//         }
//     }
// }

//====================================================================
fn SmithGGXMasking(surface_normal: Vec3, wi: Vec3, wo: Vec3, a2: f32) -> f32 {
    let dotNL: f32 = Vec3::dot(surface_normal, wi);
    let dotNV: f32 = Vec3::dot(surface_normal, wo);
    let denomC: f32 = (a2 + (1.0 - a2) * dotNV * dotNV).sqrt() + dotNV;

    return 2.0 * dotNV / denomC;
}

//====================================================================
fn SmithGGXMaskingShadowing(surface_normal: Vec3, wi: Vec3, wo: Vec3, a2: f32) -> f32 {
    let dotNL = Vec3::dot(surface_normal, wi);
    let dotNV = Vec3::dot(surface_normal, wo);

    let denomA = dotNV * (a2 + (1.0 - a2) * dotNL * dotNL).sqrt();
    let denomB = dotNL * (a2 + (1.0 - a2) * dotNV * dotNV).sqrt();

    return 2.0 * dotNL * dotNV / (denomA + denomB);
}

//====================================================================
// https://hal.archives-ouvertes.fr/hal-01509746/document
fn GgxVndf(wo: Vec3, roughness: f32, u1: f32, u2: f32) -> Vec3 {
    // -- Stretch the view vector so we are sampling as though
    // -- roughness==1
    let v: Vec3 = Vec3::new([wo.x() * roughness, wo.y(), wo.z() * roughness]).normalized();

    // -- Build an orthonormal basis with v, t1, and t2
    let t1: Vec3 = if (v.y() < 0.999) {
        (Vec3::cross(v, Vec3::Y_AXIS)).normalized()
    } else {
        Vec3::X_AXIS
    };
    let t2: Vec3 = Vec3::cross(t1, v);

    // -- Choose a point on a disk with each half of the disk weighted
    // -- proportionally to its projection onto direction v
    let a: f32 = 1.0 / (1.0 + v.y());
    let r: f32 = u1.sqrt();
    let phi: f32 = if u2 < a {
        (u2 / a) * PI
    } else {
        PI + (u2 - a) / (1.0 - a) * PI
    };
    let p1: f32 = r * phi.cos();
    let p2: f32 = r * phi.sin() * (if u2 < a { 1.0 } else { v.y() });

    // -- Calculate the normal in this stretched tangent space
    let n: Vec3 = p1 * t1 + p2 * t2 + (f32::max(0.0, 1.0 - p1 * p1 - p2 * p2)).sqrt() * v;

    // -- unstretch and normalize the normal
    return (Vec3::new([roughness * n.x(), f32::max(0.0, n.y()), roughness * n.z()])).normalized();
}

// https://schuttejoe.github.io/post/ggximportancesamplingpart1/
//====================================================================
fn ImportanceSampleGgxVdn(
    cast_result: &CastResult,
    surface_normal: Vec3,
    specularColor: Vec3,
    roughness: f32,
    wg: Vec3,
    wo: Vec3,
    mut wi: Vec3,
) -> Vec3 //reflectance
{
    let NdotWI = Vec3::dot(surface_normal, wi);

    let a = roughness;
    let a2 = a * a;

    let r0 = rand01(); // Random::MersenneTwisterFloat(twister);
    let r1 = rand01(); // Random::MersenneTwisterFloat(twister);
    let wm: Vec3 = GgxVndf(wo, roughness, r0, r1);

    wi = reflect(wm, wo);

    if (NdotWI > 0.0) {
        let F: Vec3 = schlick_fresnel(specularColor, Vec3::dot(wi, wm));
        let G1: f32 = SmithGGXMasking(surface_normal, wi, wo, a2);
        let G2: f32 = SmithGGXMaskingShadowing(surface_normal, wi, wo, a2);

        let reflectance = F * (G2 / G1);
        return reflectance;
    } else {
        let reflectance = Vec3::ZERO;
        return reflectance;
    }
}
