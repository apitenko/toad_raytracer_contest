use crate::{
    constants::COLOR_SKY_BLUE,
    math::{Mat44, Vec3},
    primitives::{mesh::Mesh, quad::Quad, triangle::Triangle, bounding_box::BoundingBox},
};

use super::{
    lights::directional::DirectionalLight, material::Material, scene::Scene, texture::Texture,
};

pub fn add_scene_defaults(scene: &mut Scene) -> anyhow::Result<()> {
    // Default directional light
    if scene.lights.len() == 0 {
        println!("No lights found, adding default Directional");
        scene.lights.push(Box::new(DirectionalLight::new(
            Vec3::new([0.5, -1.0, 0.0]),
            20000.0,
            COLOR_SKY_BLUE,
        )));
    }

    // Plane
    if false {
        println!("Adding default plane");
        // let aabb = BoundingBox::new(
        //     Quad::DEFAULT_GEOMETRY[0],
        //     Quad::DEFAULT_GEOMETRY[2],
        // );
        // let bounding_sphere = aabb.bounding_sphere();

        let color_texture = Texture::make_default_texture()?;
        let color_albedo = scene.material_storage.push_texture(color_texture);
        let mat = Material {
            color_factor: Vec3::ONE,
            color_albedo,
            ..Default::default()
        };

        let mat_shared = scene.material_storage.push_material(mat);
        let translation_matrix = Mat44::from_translation([0.0, -0.2, 0.0]);

        scene.push_triangle(Triangle::from_vertices(
            translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[0]),
            translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[1]),
            translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[2]),
            mat_shared.clone()
        ));
        scene.push_triangle(Triangle::from_vertices(
            translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[0]),
            translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[2]),
            translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[3]),
            mat_shared.clone()
        ));
    }
    Ok(())
}
