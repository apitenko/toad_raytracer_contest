use crate::{primitives::{mesh::{BoundingBox, Mesh}, quad::Quad, triangle::Triangle}, math::{Vec3, Mat44}, constants::COLOR_SKY_BLUE};

use super::{texture::Texture, scene::Scene, material::Material, lights::directional::DirectionalLight};



pub fn add_scene_defaults(scene: &mut Scene) -> anyhow::Result<()> {
    
    // Default directional light
    if scene.lights.len() == 0 {
        scene.lights.push(Box::new(DirectionalLight::new(
            Vec3::new([0.5, -1.0, 0.0]),
            0.01,
            COLOR_SKY_BLUE,
        )));
    }

    // Plane 
    let aabb = BoundingBox {
        min: Quad::DEFAULT_GEOMETRY[0],
        max: Quad::DEFAULT_GEOMETRY[2],
    };
    let bounding_sphere = aabb.bounding_sphere();

    let color_texture = Texture::make_default_texture()?;
    let color_albedo = scene.material_storage.push_texture(color_texture);
    let mat = Material {
        color_factor: Vec3::ONE,
        color_albedo,
        ..Default::default()
    };

    let mat_shared = scene.material_storage.push_material(mat);
    let translation_matrix = Mat44::from_translation([0.0, -1.0, 0.0]);

    scene.add_mesh(Mesh {
        aabb,
        bounding_sphere,
        material: mat_shared,
        triangles: vec![
            Triangle {
                vertices: [
                    translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[0]),
                    translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[1]),
                    translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[2]),
                ],
            },
            Triangle {
                vertices: [
                    translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[0]),
                    translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[2]),
                    translation_matrix.transform_point(Quad::DEFAULT_GEOMETRY[3]),
                ],
            },
        ],
    });

    Ok(())
}