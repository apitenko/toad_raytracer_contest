use std::f32::consts::PI;

use rand::Rng;
use raytracer_lib::generate_multisample_positions;

use crate::scene::acceleration_structure::acceleration_structure::AccelerationStructure;
use crate::scene::lights::light::attenuation_fn;
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

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(4);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 4] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub const MAX_BOUNCES: i32 = 1;
pub const MONTE_CARLO_THRESHOLD_BOUNCES: i32 = 4;
// pub const MAX_DEPTH: f32 = 20.0;

// todo: move to skybox
pub const SKYBOX_LIGHT_INTENSITY: f32 = 0.0;
pub const SKYBOX_COLOR: Vec3 = COLOR_SKY_BLUE;

pub const AMBIENT_LIGHT_INTENSITY: f32 = 0.0;
pub const AMBIENT_LIGHT_COLOR: Vec3 = COLOR_WHITE;

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

pub fn ray_cast(current_bounce: RayBounce, scene: &Scene) -> Vec3 {
    if current_bounce.current_bounces > MAX_BOUNCES {
        // stop recursion by limit
        return Vec3::ZERO;
    }
    // if current_bounce.remaining_depth < 0.00001 {
    //     return Vec3::ZERO;
    // }

    let cast_result = scene.geometry.single_cast(
        current_bounce.ray,
        current_bounce.refraction_state == RayRefractionState::TraversingAir,
    );
    if cast_result.has_missed() {
        // every miss is a skybox hit
        // miss after bounce
        return SKYBOX_COLOR * SKYBOX_LIGHT_INTENSITY;
        // let unit_direction = current_bounce.ray.direction().normalized();
        // let skybox_color = scene.skybox.sample_from_direction(unit_direction);
        // return skybox_color * current_bounce.multiplier;
    }

    // let mip: f32 = current_bounce.distance / 2.0;
    let mip: f32 = 0.0;

    let current_material = cast_result.material.get();

    let material_color = current_material.sample_albedo(&cast_result.uv, mip);
    let material_emission = current_material.sample_emission(&cast_result.uv, mip);

    let (material_roughness, material_metallic) = current_material.sample_roughness_metallic(&cast_result.uv, mip);

    // TODO: replace Specular with Metallic
    let material_specular =
        Vec3::from_f32([material_metallic, material_metallic, material_metallic, 0.0]);

    let material_normal = current_material.sample_normal(&cast_result.uv, mip);
    let surface_normal = (material_normal * cast_result.normal).normalized();
    // let surface_normal = cast_result.normal.normalized();

    // GGX
    const DO_DIRECT_LIGHTING: bool = true;
    const DO_INDIRECT_LIGHTING: bool = true;

    // Do explicit direct lighting to a random light in the scene
    let component_direct = if DO_DIRECT_LIGHTING {
        ggx_direct(
            scene,
            &cast_result,
            surface_normal,
            current_bounce.ray.direction(),
            material_color,
            material_specular,
            material_roughness,
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
            material_specular,
            material_roughness,
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

    let final_color = component_direct * component_indirect;
    let final_color =
        attenuation_fn(current_bounce.distance + cast_result.distance_traversed, final_color)
            + material_emission
            + AMBIENT_LIGHT_INTENSITY * AMBIENT_LIGHT_COLOR * material_color;
    return final_color;
}

// Cook-Torrance D term
#[inline]
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
#[inline]
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

fn get_perpendicular_vector(vector: Vec3) -> Vec3 {
    Vec3::from_f32([-vector.y(), vector.x(), vector.z(), 0.0])
    // Vec3::cross(vector, Vec3::from_f32([1.0, 1.0, 1.0, 0.0])).normalized()
    // Vec3::cross(vector, Vec3::from_f32([1.0, 0.0, 0.0, 0.0])).normalized()
}

// When using this function to sample, the probability density is:
//      pdf = D * NdotH / (4 * HdotV)
fn getGGXMicrofacet(roughness: f32, surface_normal: Vec3) -> Vec3 {
    let mut rng = rand::thread_rng();

    // Get our uniform random numbers
    let randVal: (f32, f32) = (rng.gen(), rng.gen());

    // Get an orthonormal basis from the normal
    let B: Vec3 = get_perpendicular_vector(surface_normal); // ! ??????????????
    let T: Vec3 = Vec3::cross(B, surface_normal);

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
        + surface_normal * cosThetaH;
}

fn ggx_direct(
    scene: &Scene,
    cast_result: &CastResult,
    surface_normal: Vec3,
    current_ray_direction: Vec3,
    material_color: Vec3,
    material_specular: Vec3,
    material_roughness: f32,
    current_bounce: &RayBounce,
) -> Vec3 {
    let V = -current_ray_direction;
    let N = surface_normal;
    let spec = material_specular;
    let hit = cast_result.intersection_point;

    //////
    let fn_sample_light = |light_source: &dyn Light| {
        let (distance_to_light, normal_into_light) =
            light_source.normal_from(cast_result.intersection_point);

        let L = normal_into_light;
        // Compute our lambertion term (N dot L)
        let NdotL = Vec3::dot(surface_normal, L).saturate();

        let light_intensity = light_source.get_emission(hit);
        let light_visibility = shadow_ray_visibility(light_source, scene, cast_result);

        // return light_intensity * light_visibility * NdotL * NdotL; // simple model for testing

        // Compute half vectors and additional dot products for GGX
        let H: Vec3 = (V + L).normalized();
        let NdotH = (Vec3::dot(N, H)).saturate();
        let LdotH = (Vec3::dot(L, H)).saturate();
        let NdotV = (Vec3::dot(N, V)).saturate();

        // Evaluate terms for our GGX BRDF model
        let D = ggx_normal_distribution(NdotH, material_roughness);
        let G = ggx_schlick_masking_term(NdotL, NdotV, material_roughness);
        let F: Vec3 = schlick_fresnel(spec, LdotH);

        // Evaluate the Cook-Torrance Microfacet BRDF model
        //     Cancel NdotL here to avoid catastrophic numerical precision issues.
        let ggxTerm: Vec3 = D * G * F / (4.0 * NdotV/* * NdotL */);

        // Compute our final color (combining diffuse lobe plus specular GGX lobe)
        return light_visibility
            * light_intensity
            * (/* NdotL * */ggxTerm + NdotL * material_color / PI);
    };

    if current_bounce.monte_carlo_reached() {
        let random_light = {
            // Pick a random light from our scene to shoot a shadow ray towards
            let lights_count = scene.lights.len();
            let random_light_index = rand::thread_rng().gen_range(0..lights_count);
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
            cast_result.intersection_point,
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
    material_specular: Vec3,
    material_roughness: f32,
) -> Vec3 {
    // ugh
    let current_ray_direction = current_bounce.ray.direction();
    let V = -current_bounce.ray.direction();
    let N = surface_normal;
    let hit = cast_result.intersection_point;

    let mut rng = rand::thread_rng();
    let (probDiffuse, diffuseMult) = probabilityToSampleDiffuse(material_color, material_specular);

    let fn_diffuse_ray = || {
        // return Vec3::ZERO;
        // Shoot a randomly selected cosine-sampled diffuse ray.
        let random_direction: Vec3 = (random_cosine_weighted_point() * N).normalized();
        let bounce_color: Vec3 = ray_cast(
            RayBounce {
                ray: Ray::new(hit, random_direction, f32::MAX),
                current_bounces: current_bounce.current_bounces + 1,
                distance: current_bounce.distance + cast_result.distance_traversed,
                refraction_state: RayRefractionState::TraversingAir,
            },
            scene,
        );

        // Accumulate the color: (NdotL * incomingLight * material_albedo / pi)
        // Probability of sampling this ray:  (NdotL / pi) * probDiffuse
        let result_color = bounce_color * material_color / diffuseMult;
        // if result_color.length() > 3.0 {
        //     println!("FUCKME IN THE ASS {:?}", probDiffuse);
        //     println!("{:?} ", result_color);
        // }
        return result_color;
    };
    let fn_specular_ray = || {
        // return Vec3::ZERO;
        // Randomly sample the NDF to get a microfacet in our BRDF
        let H: Vec3 = getGGXMicrofacet(material_roughness, N).normalized();

        // Compute outgoing direction based on this (perfectly reflective) facet
        let reflected_ray = reflect(current_ray_direction, H);

        // Compute our color by tracing a ray in this direction
        let bounce_color: Vec3 = ray_cast(
            RayBounce {
                ray: Ray::new(hit, reflected_ray, f32::MAX),
                current_bounces: current_bounce.current_bounces + 1,
                distance: current_bounce.distance + cast_result.distance_traversed,
                refraction_state: RayRefractionState::TraversingAir,
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
        // let D: f32 = ggx_normal_distribution(NdotH, rough);
        let G: f32 = ggx_schlick_masking_term(NdotL, NdotV, material_roughness);
        let F: Vec3 = schlick_fresnel(material_specular, LdotH);
        // let ggxTerm: Vec3 = D * G * F / (4.0 * NdotL * NdotV);

        // What's the probability of sampling vector H from getGGXMicrofacet()?
        // let ggxProb: f32 = (D * NdotH / (4.0 * LdotH)).saturate();
        // return ggxProb * Vec3::ONE;

        // Accumulate color:  ggx-BRDF * lightIn * NdotL / probability-of-sampling
        //    -> Note: Should really cancel and simplify the math above
        // return NdotL * bounce_color * ggxTerm / (ggxProb * (1.0 - probDiffuse));
        // return Vec3::ONE * ggxProb;

        // NOTE: this is simplified version of a line earlier
        let specular_color = bounce_color * G * F * LdotH / (NdotV * NdotH * (1.0 - diffuseMult));

        return specular_color;
    };

    if current_bounce.monte_carlo_reached() {
        let chooseDiffuse = (rng.gen::<f32>() < probDiffuse);
        if chooseDiffuse {
            return fn_diffuse_ray();
        } else {
            return fn_specular_ray();
        }
    } else {
        // const USE_MULTIPLE_DIFFUSE_RAYS: bool = true;
        // if USE_MULTIPLE_DIFFUSE_RAYS {
        //     const DIFFUSE_REFLECTIONS_NUMBER: usize = 2;
        //     let mut color_diffuse = Vec3::ZERO;
        //     for _ in 0..DIFFUSE_REFLECTIONS_NUMBER {
        //         color_diffuse += fn_diffuse_ray();
        //     }
        //     color_diffuse = color_diffuse / DIFFUSE_REFLECTIONS_NUMBER as f32;
        //     let color_specular = fn_specular_ray();
        //     return (color_diffuse + color_specular) / 2.0;
        // }
        // else {
        let color_diffuse = fn_diffuse_ray();
        let color_specular = fn_specular_ray();
        return (color_diffuse + color_specular) / 2.0;
        // }
    }
}

// TODO: find a better approach
fn probabilityToSampleDiffuse(material_albedo: Vec3, material_specular: Vec3) -> (f32,f32) {
    let lumDiffuse = material_albedo.luminosity().saturate();
    let lumSpecular = material_specular.luminosity();
    if lumSpecular > 0.01 {
        let mult = lumDiffuse / (lumDiffuse + lumSpecular);
        return (mult, mult);
    }
    else {
        return (1.0, 0.5);
    }
}

fn random_cosine_weighted_point() -> Vec3 {
    let mut rng = rand::thread_rng();
    let u = rng.gen::<f32>();
    let v = rng.gen::<f32>();

    let radial = u.sqrt();
    let theta = 2.0 * PI * v;

    let x = radial * theta.cos();
    let y = radial * theta.sin();

    return Vec3::new([x, y, (1.0 - u).sqrt()]); //??? -v ??????
}

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

    let mut rng = rand::thread_rng();
    let a = roughness;
    let a2 = a * a;

    let r0 = rng.gen::<f32>(); // Random::MersenneTwisterFloat(twister);
    let r1 = rng.gen::<f32>(); // Random::MersenneTwisterFloat(twister);
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
