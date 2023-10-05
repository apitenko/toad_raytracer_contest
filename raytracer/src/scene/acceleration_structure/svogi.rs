use crate::math::Vec3;



pub struct NodeGIInfo {
    pub directional_density: [f32; 3],
    pub directional_emittance: [Vec3; 3]
}

// preprocess:
// voxelize scene into Nodes
// bake GI into Nodes

// tracing:
// - 1x random diffuse RAY through octree for propagation
// - 8x diffuse CONES for color gathering
// - 1x standard specular reflected RAY

pub const DIFFUSE_CONES_COUNT: usize = 8;