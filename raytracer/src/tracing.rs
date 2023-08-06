use std::f32::consts::PI;

use rand::Rng;
use raytracer_lib::generate_multisample_positions;

use crate::{
    constants::MISS_COLOR_VEC3,
    math::{
        random::random_in_unit_sphere, reflect, refract, Ray, RayBounce, RayRefractionState, Vec3,
    },
    scene::{scene::Scene, lights::light::Light},
    util::fresnel_constants::FresnelConstants, primitives::cast_result::CastResult,
};

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(400);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 400] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub const MAX_BOUNCES: i32 = 50;

pub const SKYBOX_LIGHT_INTENSITY: f32 = 0.0;

// Cook-Torrance F term
fn schlick_fresnel(f0: Vec3, lDotH: f32) -> Vec3 {
    return f0 + (Vec3::new([1.0, 1.0, 1.0]) - f0) * f32::powi(1.0 - lDotH, 5);
}

// const OBJECT_REFLECTIVITY: f32 = 0.01;
fn fresnel_reflect_amount(n1: f32, n2: f32, normal: Vec3, incident: Vec3) -> f32 {
    // #if DO_FRESNEL
    // Schlick aproximation
    let mut r0: f32 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    let mut cosX: f32 = -Vec3::dot(normal, incident);
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
    let mut ret: f32 = r0 + (1.0 - r0) * x * x * x * x * x;

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

// regular cast
// RETURNS indirect_lighting
pub fn outside_cast(current_bounce: RayBounce, scene: &Scene) -> Vec3 {
    if current_bounce.bounces < 0 {
        // stop recursion by limit
        return Vec3::ZERO;
    }
    if current_bounce.multiplier < 0.00001 {
        // optional
        return Vec3::ZERO;
    }

    let cast_result = scene.geometry.single_cast(
        current_bounce.ray,
        current_bounce.refraction_state == RayRefractionState::TraversingAir,
    );
    if cast_result.is_missed() {
        // every miss is a skybox hit
        return MISS_COLOR_VEC3 * SKYBOX_LIGHT_INTENSITY;
        // let unit_direction = current_bounce.ray.direction().normalized();
        // let skybox_color = scene.skybox.sample_from_direction(unit_direction);
        // return skybox_color * current_bounce.multiplier;
    }

    let current_material = cast_result.material.get();

    let material_albedo = current_material.sample_albedo(cast_result.uv);
    let material_specular = current_material.sample_specular(cast_result.uv);
    let material_roughness = current_material.sample_roughness(cast_result.uv);
    let material_emission = current_material.sample_emission(cast_result.uv);

    // GGX
    const DO_DIRECT_LIGHTING: bool = true;
    const DO_INDIRECT_LIGHTING: bool = false;

    // Do explicit direct lighting to a random light in the scene
    let component_direct = if (DO_DIRECT_LIGHTING) {
        ggx_direct(
            scene,
            &cast_result,
            current_bounce.ray.direction(),
            material_albedo,
            material_specular,
            material_roughness,
        )
    } else {
        Vec3::ZERO
    };

    let component_indirect = if DO_INDIRECT_LIGHTING {
        // Do indirect lighting for global illumination
        // shadeColor += ggxIndirect(
        //     worldPos.xyz,
        //     worldNorm.xyz,
        //     V,
        //     difMatlColor.rgb,
        //     specMatlColor.rgb,
        //     roughness,
        //     0,
        // )
        Vec3::ZERO
    } else {
        Vec3::ZERO
    };

    // 1. Fresnel (specular/diffuse ratio)

    // let (fresnel_outside, fresnel_inside, fresnel_ratio) =
    //     if let RayRefractionState::InsideMaterial {
    //         current_outside_fresnel_coefficient,
    //     } = current_bounce.refraction_state
    //     {
    //         // ray is currently refracted inside a solid object
    //         let fresnel_inside = FresnelConstants::Air;
    //         let fresnel_outisde = current_outside_fresnel_coefficient;
    //         let fresnel_ratio = fresnel_inside / fresnel_outisde;
    //         (fresnel_inside, fresnel_outisde, fresnel_ratio)
    //     } else {
    //         // ::TraversingAir
    //         let fresnel_inside = current_material.fresnel_coefficient;
    //         let fresnel_outisde = FresnelConstants::Air;
    //         let fresnel_ratio = fresnel_outisde / fresnel_inside;
    //         (fresnel_inside, fresnel_outisde, fresnel_ratio)
    //     };

    // let specular_ratio = fresnel_reflect_amount(
    //     fresnel_inside,
    //     fresnel_outside,
    //     current_bounce.ray.direction().normalized(),
    //     cast_result.normal,
    // );

    // let diffuse_ratio = 1.0 - specular_ratio;

    // 2. Specular component is raycast

    // let component_specular = {
    //     // TODO:
    //     let material_specular = current_material.specular; // TODO: texture sample
    //     let reflected_normal = reflect(
    //         current_bounce.ray.direction().normalized(),
    //         cast_result.normal,
    //     );
    //     let rnd = random_in_unit_sphere().normalized();
    //     let reflection_normal = Vec3::lerp(rnd, reflected_normal, material_specular);

    //     let component_reflect = outside_cast(
    //         RayBounce {
    //             ray: Ray::new(cast_result.intersection_point, reflection_normal, f32::MAX),
    //             bounces: current_bounce.bounces - 1,
    //             multiplier: specular_ratio,
    //             refraction_state: current_bounce.refraction_state,
    //         },
    //         scene,
    //     );

    //     component_reflect
    // };

    // 3. Diffuse component =
    // diffuse reflectance
    // + microfacet scattering
    // + subsurface scattering
    // + volume area scattering
    // + refraction raycast
    // let component_diffuse = {};
    // 4. Emission is omnidirectional (Lambertian)


    // indirect lighting: refraction
    // let refracted_ray_direction = refract(
    //     current_bounce.ray.direction(),
    //     cast_result.normal,
    //     fresnel_ratio,
    // );

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

    // direct lighting
    // TODO: microfacets


    // for light_source in &scene.lights {
    //     let (distance_to_light, normal_into_light) =
    //         light_source.normal_from(cast_result.intersection_point);

    //     let light_cast_result = scene.geometry.single_cast(
    //         Ray::new(
    //             cast_result.intersection_point,
    //             normal_into_light,
    //             distance_to_light,
    //         ),
    //         false,
    //     );
    //     if light_cast_result.is_missed() {
    //         let light_color = light_source.get_emission(cast_result.intersection_point);

    //         let light_power = (Vec3::dot(cast_result.normal, normal_into_light)) * light_color;

    //         let color = material_albedo * light_color * light_power;

    //         direct_lighting = direct_lighting + color;
    //     }
    // }

    // TODO: Subsurface Scattering

    // ! Blend components  -------------------------
    
    let final_color = component_direct;
    return final_color * current_bounce.multiplier;
}

// cast inside object (refractive)
pub fn inside_cast() {}

// pub fn oren_nayar() {
//     // each facet is Lambertian
//     // raytrace the
//     let l1_direct_illumination;

//     let l0_microfacets;
// }

// Cook-Torrance D term
fn ggx_normal_distribution(NdotH: f32, roughness: f32) -> f32 {
    // TODO: remove clamps
    let NdotH = NdotH.clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    let roughness = roughness.clamp(f32::EPSILON, 1.0 - f32::EPSILON);

    let a2 = roughness * roughness;
    let d = ((NdotH * a2 - NdotH) * NdotH + 1.0);
    return a2 / (d * d * PI);
}

// Cook-Torrance G term
// TODO: maybe find a better model
fn ggx_schlick_masking_term(NdotL: f32, NdotV: f32, roughness: f32) -> f32 {
    // TODO: remove clamps
    let NdotL = NdotL.clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    let NdotV = NdotV.clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    let roughness = roughness.clamp(f32::EPSILON, 1.0 - f32::EPSILON);

    // Karis notes they use alpha / 2 (or roughness^2 / 2)
    let k = roughness * roughness / 2.0;

    // Compute G(v) and G(l).  These equations directly from Schlick 1994
    //     (Though note, Schlick's notation is cryptic and confusing.)
    let g_v = NdotV / (NdotV * (1.0 - k) + k);
    let g_l = NdotL / (NdotL * (1.0 - k) + k);
    return g_v * g_l;
}

// When using this function to sample, the probability density is:
//      pdf = D * NdotH / (4 * HdotV)
fn getGGXMicrofacet(roughness: f32, hitNorm: Vec3) -> Vec3 {
    let mut rng = rand::thread_rng();

    // Get our uniform random numbers
    let randVal: (f32, f32) = (rng.gen(), rng.gen());

    // Get an orthonormal basis from the normal
    let B: Vec3 = hitNorm;
    // let B: Vec3 = getPerpendicularVector(hitNorm); // ! ??????????????
    let T: Vec3 = Vec3::cross(B, hitNorm);

    // GGX NDF sampling
    let a2 = roughness * roughness;
    let cosThetaH = f32::sqrt(f32::max(
        0.0,
        (1.0 - randVal.0) / ((a2 - 1.0) * randVal.0 + 1.0),
    ));
    let sinThetaH = f32::sqrt(f32::max(0.0, 1.0 - cosThetaH * cosThetaH));
    let phiH = randVal.1 * PI * 2.0;

    // Get our GGX NDF sample (i.e., the half vector)
    return T * (sinThetaH * f32::cos(phiH))
        + B * (sinThetaH * f32::sin(phiH))
        + hitNorm * cosThetaH;
}

fn ggx_direct(
    scene: &Scene,
    cast_result: &CastResult,
    current_ray_direction: Vec3,
    material_albedo: Vec3,
    material_specular: Vec3,
    material_roughness: f32,
) -> Vec3 {
    let V = current_ray_direction;
    let dif = material_albedo;
    let N = cast_result.normal;
    let rough = material_roughness;
    let spec = material_specular;
    let hit = cast_result.intersection_point;

    let random_light = {
        // Pick a random light from our scene to shoot a shadow ray towards
        let lights_count = scene.lights.len();
        let random_light_index = rand::thread_rng().gen_range(0..lights_count);
        let random_light = scene.lights[random_light_index].as_ref();
        random_light
    };
    //////
    let light_source = random_light;
    let (distance_to_light, normal_into_light) =
    light_source.normal_from(cast_result.intersection_point);
    
    let L = normal_into_light;
    // Compute our lambertion term (N dot L)
    let NdotL = Vec3::dot(cast_result.normal, L).clamp(0.0,1.0);

    let light_intensity = random_light.get_emission(hit);
    let light_visibility = shadow_ray_visibility(light_source, scene, cast_result);

    // Compute half vectors and additional dot products for GGX
    let H: Vec3 = (V + L).normalized();
    let NdotH = (Vec3::dot(N, H)).clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    let LdotH = (Vec3::dot(L, H)).clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    let NdotV = (Vec3::dot(N, V)).clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    
    // Evaluate terms for our GGX BRDF model
    let D = ggx_normal_distribution(NdotH, rough);
    let G = ggx_schlick_masking_term(NdotL, NdotV, rough);
    let F: Vec3 = schlick_fresnel(spec, LdotH);

    // Evaluate the Cook-Torrance Microfacet BRDF model
    //     Cancel NdotL here to avoid catastrophic numerical precision issues.
    let ggxTerm: Vec3 = D * G * F / (4.0 * NdotV/* * NdotL */);

    // Compute our final color (combining diffuse lobe plus specular GGX lobe)
    return light_visibility * light_intensity * (/* NdotL * */ggxTerm + NdotL * dif / PI);
}


fn shadow_ray_visibility(light_source: &dyn Light, scene: &Scene, cast_result: &CastResult) -> Vec3 {

    let (distance_to_light, normal_into_light) =
    light_source.normal_from(cast_result.intersection_point);

    let light_cast_result = scene.geometry.single_cast(
        Ray::new(
            cast_result.intersection_point,
            normal_into_light,
            distance_to_light,
        ),
        false,
    );
    
    if !light_cast_result.is_missed() {
        return Vec3::ZERO;
    }
    else {
        return Vec3::ONE;
    }
}

// fn probabilityToSampleDiffuse( difColor: Vec3, specColor: Vec3) -> f32
// {
// 	let lumDiffuse = f32::max(0.01, (difColor.rgb));
// 	let lumSpecular = f32::max(0.01, (specColor.rgb));
// 	return lumDiffuse / (lumDiffuse + lumSpecular);
// }