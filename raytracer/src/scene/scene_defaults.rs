use crate::primitives::quad::QUAD_DEFAULT_GEOMETRY;
use crate::scene::material::IMaterialStorage;
use crate::{
    constants::COLOR_SKY_BLUE,
    math::{Mat44, Vec3},
    primitives::triangle::Triangle,
    scene::texture::sampler::{MagFilter, MinFilter, Sampler},
};

use super::{
    lights::directional::DirectionalLight, material::Material, scene::Scene,
    texture::texture::Texture,
};

pub fn add_scene_defaults(scene: &mut Scene) -> anyhow::Result<()> {
    // Default directional light
    if scene.lights.len() == 0 {
        println!("No lights found, adding default Directional");
        scene.lights.push(Box::new(DirectionalLight::new(
            Vec3::new([0.5, -1.0, 0.0]),
            0.1,
            COLOR_SKY_BLUE,
        )));
    }

    // Plane
    // if false {
    //     println!("Adding default plane");
    //     // let aabb = BoundingBox::new(
    //     //     Quad::DEFAULT_GEOMETRY[0],
    //     //     Quad::DEFAULT_GEOMETRY[2],
    //     // );
    //     // let bounding_sphere = aabb.bounding_sphere();

    //     let color_texture = Texture::make_default_texture()?;
    //     // let color_albedo = scene.material_storage.push_texture(color_texture);
    //     let color_albedo = Sampler::new(
    //         &mut scene.material_storage,
    //         color_texture,
    //         MinFilter::Nearest,
    //         MagFilter::Nearest,
    //         0,
    //     );
    //     let mat = Material {
    //         color_factor: Vec3::ONE,
    //         color_texture: color_albedo,
    //         ..scene.default_material.get().clone()
    //     };

    //     let mat_shared = scene.material_storage.push_material(mat);
    //     let translation_matrix = Mat44::from_translation([0.0, -0.2, 0.0]);

    //     scene.push_triangle(Triangle::from_vertices(
    //         translation_matrix.transform_point(QUAD_DEFAULT_GEOMETRY[0]),
    //         translation_matrix.transform_point(QUAD_DEFAULT_GEOMETRY[1]),
    //         translation_matrix.transform_point(QUAD_DEFAULT_GEOMETRY[2]),
    //         mat_shared.clone(),
    //     ));
    //     scene.push_triangle(Triangle::from_vertices(
    //         translation_matrix.transform_point(QUAD_DEFAULT_GEOMETRY[0]),
    //         translation_matrix.transform_point(QUAD_DEFAULT_GEOMETRY[2]),
    //         translation_matrix.transform_point(QUAD_DEFAULT_GEOMETRY[3]),
    //         mat_shared.clone(),
    //     ));
    // }
    Ok(())
}
