use raytracer_lib::generate_multisample_positions;

use crate::{
    constants::MISS_COLOR_VEC3,
    math::{
        random::random_in_unit_sphere, reflect, refract, Ray, RayBounce, RayRefractionState, Vec3,
    },
    scene::scene::Scene,
    util::fresnel_constants::FresnelConstants,
};

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(8);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 8] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub const MAX_BOUNCES: i32 = 50;

pub const SKYBOX_LIGHT_INTENSITY: f32 = 0.0;

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

    let cast_result = scene.geometry.single_cast(current_bounce.ray);
    if cast_result.is_missed() {
        // every miss is a skybox hit
        return MISS_COLOR_VEC3 * SKYBOX_LIGHT_INTENSITY;
        // let unit_direction = current_bounce.ray.direction().normalized();
        // let skybox_color = scene.skybox.sample_from_direction(unit_direction);
        // return skybox_color * current_bounce.multiplier;
    }

    let mut direct_lighting = Vec3::ZERO;

    // TODO: microfacets?
    // indirect lighting: reflection
    let material_specular = cast_result.material.get().specular; // TODO: sample from material

    let reflected_normal = reflect(
        current_bounce.ray.direction().normalized(),
        cast_result.normal,
    );
    let rnd = random_in_unit_sphere().normalized();
    let reflection_normal = Vec3::lerp(rnd, reflected_normal, material_specular);

    let (fresnel_outside, fresnel_inside, fresnel_ratio) =
        if let RayRefractionState::InsideMaterial {
            current_outside_fresnel_coefficient,
        } = current_bounce.refraction_state
        {
            // ray is currently refracted inside a solid object
            let fresnel_inside = FresnelConstants::Air;
            let fresnel_outisde = current_outside_fresnel_coefficient;
            let fresnel_ratio = fresnel_inside / fresnel_outisde;
            (fresnel_inside, fresnel_outisde, fresnel_ratio)
        } else {
            // ::TraversingAir
            let fresnel_inside = cast_result.material.get().fresnel_coefficient;
            let fresnel_outisde = FresnelConstants::Air;
            let fresnel_ratio = fresnel_outisde / fresnel_inside;
            (fresnel_inside, fresnel_outisde, fresnel_ratio)
        };

    let reflect_multiplier = fresnel_reflect_amount(
        fresnel_inside,
        fresnel_outside,
        current_bounce.ray.direction().normalized(),
        cast_result.normal,
    );

    let refract_multiplier = 1.0 - reflect_multiplier;

    let component_reflect = outside_cast(
        RayBounce {
            ray: Ray::new(cast_result.intersection_point, reflection_normal, f32::MAX),
            bounces: current_bounce.bounces - 1,
            multiplier: reflect_multiplier,
            refraction_state: RayRefractionState::TraversingAir,
        },
        scene,
    );
    // Split energy between Lost and Refracted
    let lost_multiplier = 0.0;
    let refracted_multiplier = 1.0 - lost_multiplier;
    // TODO: subsurface scattering / BSSRDF

    let component_refract = {
        // indirect lighting: refraction
        let refracted_ray_direction = refract(
            current_bounce.ray.direction(),
            cast_result.normal,
            fresnel_ratio,
        );
    
        if let RayRefractionState::TraversingAir = current_bounce.refraction_state {
            outside_cast(
                // TODO: should be inside cast
                RayBounce {
                    ray: Ray::new(
                        cast_result.intersection_point,
                        refracted_ray_direction,
                        f32::MAX,
                    ),
                    bounces: current_bounce.bounces - 1,
                    multiplier: refracted_multiplier,
                    refraction_state: RayRefractionState::InsideMaterial {
                        current_outside_fresnel_coefficient: fresnel_outside,
                    },
                },
                scene,
            )
        } else {
            outside_cast(
                // TODO: should be inside cast
                RayBounce {
                    ray: Ray::new(
                        cast_result.intersection_point,
                        refracted_ray_direction,
                        f32::MAX,
                    ),
                    bounces: current_bounce.bounces - 1,
                    multiplier: refracted_multiplier,
                    refraction_state: RayRefractionState::TraversingAir,
                },
                scene,
            )
        }
    };

    // direct lighting
    // TODO: microfacets

    let material_albedo = {
        let material_color_tint = cast_result.material.get().color_tint;
        let (u, v) = cast_result.uv;
        let u = (u * cast_result.material.get().uv_scale).fract();
        let v = (v * cast_result.material.get().uv_scale).fract();
        material_color_tint * cast_result.material.get().albedo.get().sample(u, v)
    };

    for light_source in &scene.lights {
        let (distance_to_light, normal_into_light) =
            light_source.normal_from(cast_result.intersection_point);

        let light_cast_result = scene.geometry.single_cast(Ray::new(
            cast_result.intersection_point,
            normal_into_light,
            distance_to_light,
        ));
        if light_cast_result.is_missed() {
            let light_color = light_source.get_emission(cast_result.intersection_point);

            let light_power = (Vec3::dot(cast_result.normal, normal_into_light)) * light_color;

            let color = light_color * light_power;

            direct_lighting = direct_lighting + color;
        }
    }

    // TODO: Subsurface Scattering
    let component_diffuse = direct_lighting * (1.0 - material_specular) * material_albedo;
    let component_specular = component_reflect * material_specular * material_albedo;

    let indirect_lighting = (component_refract) * refract_multiplier
        + (component_diffuse + component_specular);
    indirect_lighting * current_bounce.multiplier
}

// cast inside object (refractive)
pub fn inside_cast() {}
