use raytracer_lib::generate_multisample_positions;

use crate::{
    constants::MISS_COLOR_VEC3,
    math::{random::random_in_unit_sphere, reflect, Ray, RayBounce, RayRefractionState, Vec3},
    scene::scene::Scene,
    util::fresnel_constants::FresnelConstants,
};

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(4);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 4] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub const MAX_BOUNCES: i32 = 5;

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
    if current_bounce.multiplier < f32::EPSILON {
        // optional
        return Vec3::ZERO;
    }

    let cast_result = scene.geometry.single_cast(current_bounce.ray);
    if cast_result.is_missed() {
        // every miss is a skybox hit
        return MISS_COLOR_VEC3;
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

    let reflect_multiplier = if let RayRefractionState::InsideMaterial {
        current_outside_fresnel_coefficient,
    } = current_bounce.refraction_state
    {
        // ray is currently refracted inside a solid object
        fresnel_reflect_amount(
            FresnelConstants::Air,
            current_outside_fresnel_coefficient,
            current_bounce.ray.direction().normalized(),
            cast_result.normal,
        )
    } else {
        // ::TraversingAir
        fresnel_reflect_amount(
            cast_result.material.get().fresnel_coefficient,
            FresnelConstants::Air,
            current_bounce.ray.direction().normalized(),
            cast_result.normal,
        )
    };

    let refract_multiplier = 1.0 - reflect_multiplier;

    let specular_indirect_lighting = outside_cast(
        RayBounce {
            ray: Ray::new(cast_result.intersection_point, reflection_normal, f32::MAX),
            bounces: current_bounce.bounces - 1,
            multiplier: reflect_multiplier,
            refraction_state: RayRefractionState::TraversingAir,
        },
        scene,
    );
    // indirect lighting: refraction

    // direct lighting
    // TODO: microfacets

    let material_color_tint = cast_result.material.get().color_tint;

    let (u, v) = cast_result.uv;
    let material_albedo =
        material_color_tint * cast_result.material.get().albedo.get().sample(u, v);

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

            let color = material_albedo * light_color * light_power;

            direct_lighting = direct_lighting + color;
        }
    }

    // TODO: Subsurface Scattering
    let indirect_lighting =
        direct_lighting * (1.0 - material_specular) * refract_multiplier + specular_indirect_lighting * material_albedo;
    indirect_lighting * current_bounce.multiplier
}

// cast inside object (refractive)
pub fn inside_cast() {}
