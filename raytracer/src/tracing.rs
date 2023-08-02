use raytracer_lib::generate_multisample_positions;

use crate::{
    math::{random::random_in_unit_sphere, reflect, Ray, RayBounce, Vec3},
    scene::scene::Scene,
};

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(400);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 400] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub const MAX_BOUNCES: i32 = 50;

const OBJECT_REFLECTIVITY: f32 = 0.01;
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
    ret = (OBJECT_REFLECTIVITY + (1.0 - OBJECT_REFLECTIVITY) * ret);
    return ret;
    // #else
    // 	return OBJECT_REFLECTIVITY;
    // #endif
}

// refractive index of some common materials:
// http://hyperphysics.phy-astr.gsu.edu/hbase/Tables/indrf.html
const REFRACTIVE_INDEX_OUTSIDE: f32 = 1.00029;
const REFRACTIVE_INDEX_INSIDE: f32 = 1.125;

// regular cast
// RETURNS indirect_lighting
pub fn outside_cast(current_bounce: RayBounce, scene: &Scene) -> Vec3 {
    if current_bounce.bounces < 0 {
        // stop recursion by limit
        return Vec3::ZERO;
    }
    if current_bounce.multiplier < 0.001 {
        return Vec3::ZERO;
    }

    let cast_result = scene.geometry.single_cast(current_bounce.ray);
    if cast_result.is_missed() {
        return Vec3::ZERO;
    }

    let mut direct_lighting = Vec3::ZERO;

    // TODO: microfacets?
    // indirect lighting: reflection
    let specular_power = cast_result.material.get().specular_power; // TODO: sample from material

    let reflected_normal = reflect(
        current_bounce.ray.direction().normalized(),
        cast_result.normal,
    );
    let rnd = random_in_unit_sphere().normalized();
    let reflection_normal = Vec3::lerp(rnd, reflected_normal, specular_power);

    let reflect_multiplier = fresnel_reflect_amount(
        REFRACTIVE_INDEX_INSIDE,  // TODO: sample from material
        REFRACTIVE_INDEX_OUTSIDE, // TODO: sample from material
        current_bounce.ray.direction().normalized(),
        cast_result.normal,
    );
    let refract_multiplier = 1.0 - reflect_multiplier;

    let specular_indirect_lighting = outside_cast(
        RayBounce {
            ray: Ray::new(cast_result.intersection_point, reflection_normal, f32::MAX),
            bounces: current_bounce.bounces - 1,
            multiplier: reflect_multiplier,
        },
        scene,
    );
    // indirect lighting: refraction

    // direct lighting
    // TODO: microfacets
    
    let material_tint = cast_result.material.get().color;
    let (u,v) = cast_result.uv;
    let material_albedo = material_tint * cast_result.material.get().texture.get().sample(u,v);

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
    let indirect_lighting = direct_lighting * (1.0 - specular_power) + specular_indirect_lighting * material_albedo;
    indirect_lighting
}

// cast inside object (refractive)
pub fn inside_cast() {}
