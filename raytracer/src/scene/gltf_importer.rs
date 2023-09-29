use std::{
    f32::consts::PI,
    path::{Path, PathBuf}, iter,
};
use base64::Engine;
use gltf::{buffer, camera::Projection, image, scene::Transform, Document, Gltf};

use crate::{
    math::{Mat44, Vec3},
    primitives::{self, mesh::Mesh, sphere::Sphere, triangle::Triangle, bounding_box::BoundingBox},
};
use itertools::Itertools;

use super::{
    camera::Camera,
    lights::{
        directional::DirectionalLight,
        point::{PointLight, PointLightRadius}, spot::{SpotLight, SpotLightRange},
    },
    material::{Material, MaterialShared},
    scene::Scene,
    texture::Texture,
    uri::{resolve_uri, UriResolved},
};

struct ImportedGltfScene {
    document: Document,
    buffers: Vec<buffer::Data>,
    images: Vec<image::Data>,
}

type GltfImport = (Document, Vec<buffer::Data>, Vec<image::Data>);
impl From<GltfImport> for ImportedGltfScene {
    fn from(value: GltfImport) -> Self {
        Self {
            document: value.0,
            buffers: value.1,
            images: value.2,
        }
    }
}

pub fn read_into_scene(app_scene: &mut Scene, path: &str) -> anyhow::Result<()> {
    let imported: ImportedGltfScene = {
        println!("gltf::import start");
        let time_start = std::time::Instant::now();
        let imported = gltf::import(path)?.into();
        let time_spent = std::time::Instant::now() - time_start;
        println!("gltf::import successful, took {:?}", time_spent);
        imported
    };

    let mut gltf_root_folder = PathBuf::from(path);
    gltf_root_folder.pop();
    // let gltf = Gltf::open("./res/sponza/sponza_json_all.gltf")?;
    // 1. Find gltf camera

    let scene = imported
        .document
        .default_scene()
        .expect("No default scene in gtlf file");

    let mut cameras = imported.document.cameras();
    let first_camera = cameras.next();
    let (camera_view_matrix, camera_projection_matrix, aspect_ratio) = match first_camera {
        None => {
            println!("Camera not found; using default");
            let aspect_ratio = 4.0 / 3.0;
            let camera_view_matrix: Mat44 = Mat44::IDENTITY;
            let camera_projection_matrix: Mat44 =
                Mat44::from_perspective_rh(85.0_f32.to_radians(), 4.0 / 3.0, 0.01, 100.0);

            (camera_view_matrix, camera_projection_matrix, aspect_ratio)
        }
        Some(_camera_found) => {
            let (transform, camera) = scan_for_camera(Mat44::IDENTITY, &mut scene.nodes())
                .expect("No camera found in the gltf scene: something's wrong with the code");
            println!("Camera found");
            let camera_view_matrix: Mat44 = transform.inverse();
            let (camera_projection_matrix, aspect_ratio) = from_gltf_projection(camera.projection());

            (camera_view_matrix, camera_projection_matrix, aspect_ratio)
        }
    };

    app_scene.set_camera(Camera::from_matrices(
        camera_view_matrix,
        camera_projection_matrix,
    ));

    app_scene.set_aspect_ratio(aspect_ratio);

    // 2. Import all vertices into the acceleration structure, applying camera transform

    // let test_point = Vec3::from_f32([0.0, 0.0, -5.0, 1.0]);
    // let transformed_test_point = camera_view_matrix.transform_point(test_point);
    // let test_point = view_projection.transform_point(test_point);
    // let test_point = view_projection.transform_point(test_point);
    // println!("{:?}", test_point.divided_by_w());

    for node in scene.nodes() {
        import_node(app_scene, &node, &Mat44::IDENTITY, &imported, &gltf_root_folder)?;
        // println!(
        //     "Node #{} has {} children",
        //     node.index(),
        //     node.children().count(),
        // );
    }

    Ok(())
}

pub fn scan_for_camera<'a>(
    parent_transform: Mat44,
    nodes: &mut dyn Iterator<Item = gltf::Node<'a>>,
) -> Option<(Mat44, gltf::Camera<'a>)> {
    for child in &mut *nodes {
        let current_transform: Mat44 = child.transform().into();
        let accumulated_transform = current_transform * parent_transform;
        match scan_for_camera(
            accumulated_transform,
            // current_transform,
            &mut child.children(),
        ) {
            Some(transform_plus_camera) => return Some(transform_plus_camera),
            None => (),
        }
        match child.camera() {
            Some(cam) => {
                return Some((accumulated_transform, cam));
            }
            None => {}
        }
    }
    return None;
}

#[derive(Clone, Debug)]
struct GltfImportError {
    cause: String,
}

impl GltfImportError {
    pub fn new(cause: String) -> Self {
        Self { cause }
    }
}
impl std::fmt::Display for GltfImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl std::error::Error for GltfImportError {}

fn import_node(
    app_scene: &mut Scene,
    node: &gltf::Node,
    parent_transform: &Mat44,
    imported: &ImportedGltfScene,
    gltf_folder: &Path,
) -> anyhow::Result<()> {
    let own_transform: Mat44 = node.transform().into();
    let accumulated_transform = *parent_transform * own_transform;

    match node.mesh() {
        Some(mesh) => {
            for primitive in mesh.primitives() {
                let material = import_material(app_scene, imported, gltf_folder, primitive.material())?;

                let bbox = primitive.bounding_box();
                let positions = primitive
                    .get(&gltf::Semantic::Positions)
                    .expect("No positions for primitive");
                let tex_coords = primitive
                    .get(&gltf::Semantic::TexCoords(0))
                    .expect("No texcoord (channel 0) for primitive");
                let normals = primitive
                    .get(&gltf::Semantic::Normals)
                    .expect("No normals for primitive");

                let reader = primitive.reader(|buffer| Some(&imported.buffers[buffer.index()]));

                let index_iter = {
                    let indices = reader.read_indices();
                    let indices: Box<dyn Iterator<Item = u32>> = match indices {
                        Some(indices) => Box::new(indices.into_u32()),
                        None => Box::new(0..),
                    };
                    indices
                };

                let index_iter = index_iter.tuple_windows().step_by(3);
                
                // let uv_iter: Box<dyn Iterator<Item = [f32;2]>> = {
                //     let uv = reader.read_tex_coords(0);
                //     let uv: Box<dyn Iterator<Item = [f32;2]>> = match uv {
                //         Some(uv) => Box::new(uv.into_f32()),
                //         None => Box::new(iter::repeat([0.5f32, 0.5])),
                //     };
                //     uv
                // };
                // let uv_iter = uv_iter.tuple_windows().step_by(3);

                let mode_ = primitive.mode();

                if let gltf::mesh::Mode::Triangles = mode_ {
                    // should be empty here
                } else {
                    return Err(
                        GltfImportError::new(format!("Wrong mesh mode: {:?}", mode_)).into(),
                    );
                }

                let positions_reader = match reader.read_positions() {
                    None => return Err(GltfImportError::new("No positions found".into()).into()),
                    Some(p) => p,
                };

                


                let input_positions: Vec<_> = positions_reader.collect();

                let input_uv: Vec<_> = {
                    let uv_reader = reader.read_tex_coords(0);
                    match uv_reader {
                        None => {
                            (0..input_positions.len()/2).map(|huynya| {
                                [huynya as f32, huynya as f32]
                            }).collect()
                        },
                        Some(uv_reader) => {
                            uv_reader.into_f32().collect()
                        }
                    }
                };

                let read_normal_for_triangle: Box<dyn Fn(usize, Mat44, Vec3) -> Vec3> = {
                    let normals_reader = reader.read_normals();
                    match normals_reader {
                        None => {
                            let boxed_closure = Box::new(|index: usize, inv_tr_mat: Mat44, fallback_geometry_normal: Vec3| {
                                fallback_geometry_normal
                            });
                            boxed_closure
                        },
                        Some(fuckme) => {
                            let data: Vec<_> = fuckme.collect();
                            
                            let boxed_closure = Box::new(move |index: usize, inv_tr_mat: Mat44, fallback_geometry_normal: Vec3| {
                                let normal = Vec3::from_f32_3(data[index], 0.0);
                                let normal = inv_tr_mat * normal;
                                normal
                            });
                            boxed_closure
                        }
                    }
                };

                assert!(input_uv.len() == input_positions.len());

                let inverse_transposed_matrix = accumulated_transform.inverse().transposed();
                // let mut final_positions = Vec::with_capacity(input_positions.len() * 2); // guesstimating the total size

                for ((i0, i1, i2)) in index_iter {
                    // transform position
                    let p0 = accumulated_transform
                        .transform_point(Vec3::from_f32_3(input_positions[i0 as usize], 1.0));
                    let p1 = accumulated_transform
                        .transform_point(Vec3::from_f32_3(input_positions[i1 as usize], 1.0));
                    let p2 = accumulated_transform
                        .transform_point(Vec3::from_f32_3(input_positions[i2 as usize], 1.0));

                    // transform normals
                    let fallback_geometry_normal = Vec3::calculate_normal_from_points(p0, p1, p2);
                    
                    let n0 = read_normal_for_triangle(i0 as usize, inverse_transposed_matrix, fallback_geometry_normal);
                    let n1 = read_normal_for_triangle(i1 as usize, inverse_transposed_matrix, fallback_geometry_normal);
                    let n2 = read_normal_for_triangle(i2 as usize, inverse_transposed_matrix, fallback_geometry_normal);

                    let uv0 = input_uv[i0 as usize];
                    let uv1 = input_uv[i1 as usize];
                    let uv2 = input_uv[i2 as usize];

                    app_scene.push_triangle(Triangle {
                        vertices: [p0, p1, p2],
                        uv: [uv0, uv1, uv2],
                        normals: [n0, n1, n2],
                        material: material.clone()
                    });
                }

                // let aabb = BoundingBox::from_gltf(primitive.bounding_box());
                // let bounding_sphere = aabb.bounding_sphere();

                // app_scene.add_mesh(Mesh {
                //     triangles: final_positions,
                //     aabb,
                //     bounding_sphere,
                //     material,
                // })
            }
        }
        None => (),
    }

    match node.light() {
        None => (),
        Some(light) => {
            let color = Vec3::new(light.color());
            let intensity = light.intensity();
            let direction = accumulated_transform * Vec3::from_f32([0.0, 0.0, -1.0, 0.0]);
            let position = accumulated_transform * Vec3::from_f32([0.0, 0.0, 0.0, 1.0]);

            match light.kind() {
                gltf::khr_lights_punctual::Kind::Directional => {
                    app_scene.lights.push(Box::new(DirectionalLight {
                        color,
                        intensity,
                        direction,
                    }))
                }
                gltf::khr_lights_punctual::Kind::Point => match light.range() {
                    None => app_scene.lights.push(Box::new(PointLight {
                        color,
                        intensity,
                        position,
                    })),
                    Some(range) => app_scene.lights.push(Box::new(PointLightRadius {
                        color,
                        intensity,
                        position,
                        radius: range,
                    })),
                },
                gltf::khr_lights_punctual::Kind::Spot {
                    inner_cone_angle,
                    outer_cone_angle,
                } => match light.range() {
                    None => app_scene.lights.push(Box::new(SpotLight {
                        color,
                        intensity,
                        position,
                        inner_cone_angle,
                        outer_cone_angle,
                    })),
                    Some(range) => app_scene.lights.push(Box::new(SpotLightRange {
                        color,
                        intensity,
                        position,
                        inner_cone_angle,
                        outer_cone_angle,
                        range
                    })),
                }
            }
        }
    }

    for child in node.children() {
        import_node(app_scene, &child, &accumulated_transform, imported, gltf_folder)?;
    }

    Ok(())
}

impl From<Transform> for Mat44 {
    fn from(transform: Transform) -> Self {
        let matrix = match transform {
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Mat44::from_decomposed(translation, rotation, scale),
            gltf::scene::Transform::Matrix { matrix } => Mat44::from_4x4(matrix),
        };
        return matrix;
    }
}


pub fn from_gltf_projection(projection: Projection) -> (Mat44, f32) {
    match projection {
        gltf::camera::Projection::Orthographic(ortho) => {
            let near = ortho.znear();
            let far = ortho.zfar();
            let xmag = ortho.xmag();
            let ymag = ortho.ymag();

            return (
                Mat44::from_orthographic(xmag, ymag, near, far), 
                1.0
            )
        }
        gltf::camera::Projection::Perspective(persp) => {
            let yfov = persp.yfov();
            let aspect_ratio = persp.aspect_ratio().unwrap_or(1.0);
            let near = persp.znear();
            let far_option = persp.zfar();

            match far_option {
                Some(far) => {
                    // Far plane exist
                    // GLTF default is Right Handed (forward is -z)
                    return (Mat44::from_perspective_rh(yfov, 1.0 / aspect_ratio, near, far), aspect_ratio)
                }
                None => {
                    // Infinite far
                    return (Mat44::from_perspective_infinite(yfov, 1.0 / aspect_ratio, near), aspect_ratio)
                }
            }
        }
    }
}


fn import_material(
    app_scene: &mut Scene,
    imported: &ImportedGltfScene,
    gltf_folder: &Path,
    material: gltf::material::Material,
) -> anyhow::Result<MaterialShared> {
    
    let pbr_info = material.pbr_metallic_roughness();
    let color_factor = pbr_info.base_color_factor();
    let metallic_factor = pbr_info.metallic_factor();
    let color_texture = match pbr_info.base_color_texture() {
        None => Texture::make_default_texture()?,
        Some(t) => {
            let texture_uv_index = t.tex_coord();
            if texture_uv_index != 0 {
                todo!("texture_uv_index != 0; it is {}", texture_uv_index);
            }
            let texture = t.texture();
            let sampler = texture.sampler();
            let image = texture.source();
            // todo: sampler

            let texture = match image.source() {
                image::Source::Uri { uri, mime_type } => match resolve_uri(uri)? {
                    UriResolved::Base64(base64_slice) => Texture::new_from_base64_str(base64_slice),
                    UriResolved::Filename(filename) => {
                        let resolved_path = gltf_folder.join(filename);
                        let read_data = std::fs::read(resolved_path)?;
                        Texture::new_from_raw_bytes(&read_data)
                    },
                    _ => {
                        panic!("not implemented")
                    }
                },
                image::Source::View { view, mime_type } => {
                    if let Some(_) = view.stride() {
                        todo!("stride is not supported");
                    }
                    let buffer = view.buffer();

                    let texture = match buffer.source() {
                        buffer::Source::Bin => {
                            
                            let buffer_data = &imported.buffers[buffer.index()];
                            let offset = view.offset();
                            let length = view.length();
                            Texture::new_from_raw_bytes(&buffer_data.0[offset..offset+length])
                        }
                        buffer::Source::Uri(uri) => match resolve_uri(uri)? {
                            UriResolved::Base64(base64_str) => {

                                let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
                                    .decode(&base64_str[8..])?;

                                let offset = view.offset();
                                let length = view.length();
                                Texture::new_from_raw_bytes(&bytes[offset..offset+length])
                            },
                            UriResolved::Filename(filename) => {
                                let resolved_path = gltf_folder.join(filename);
                                let read_data = std::fs::read(resolved_path)?;
                                Texture::new_from_raw_bytes(&read_data)
                            },
                            _ => {
                                panic!("non-base64 uri not implemented")
                            }
                        },
                    };

                    texture
                }
            };

            texture?
        }
    };

    let color_texture = app_scene.material_storage.push_texture(color_texture);

    let mat = Material {
        color_factor: Vec3::from_f32(color_factor),
        color_albedo: color_texture,
        // metallic_factor,
        ..Default::default()
    };

    let mat_shared = app_scene.material_storage.push_material(mat);
    Ok(mat_shared)
}

