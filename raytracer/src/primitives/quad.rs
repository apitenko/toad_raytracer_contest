use crate::{
    math::Vec3,
    scene::material::{Material, MaterialShared},
};

use super::{cast_result::CastResult, shape::Shape};

pub struct Quad {
    material: MaterialShared,
}

const UV_SCALE: f32 = 20.0;
const PLANE_SIZE: f32 = 50.0;

impl Quad {
    pub fn new(material: MaterialShared) -> Self {
        Self { material }
    }

    const POINTS: [Vec3; 4] = [
        // ccw
        Vec3::new([PLANE_SIZE, -1.0, -PLANE_SIZE]),
        Vec3::new([PLANE_SIZE, -1.0, PLANE_SIZE]),
        Vec3::new([-PLANE_SIZE, -1.0, PLANE_SIZE]),
        Vec3::new([-PLANE_SIZE, -1.0, -PLANE_SIZE]),
    ];
}

impl Shape for Quad {
    fn intersect(&self, ray: crate::math::Ray) -> Option<CastResult> {
        let a = Self::POINTS[1] - Self::POINTS[0];
        let b = Self::POINTS[3] - Self::POINTS[0];
        let c = Self::POINTS[2] - Self::POINTS[0];
        let p = ray.origin() - Self::POINTS[0];

        let nor = Vec3::cross(a, b);
        let t: f32 = -Vec3::dot(p, nor) / Vec3::dot(ray.direction(), nor);

        if t < 0.0 {
            // MISS
            return None;
        }

        // intersection point
        let pos = p + t * ray.direction();

        // select projection plane
        let mor = Vec3::abs(&nor);

        let id: usize = if mor.x() > mor.y() && mor.x() > mor.z() {
            0
        } else {
            if mor.y() > mor.z() {
                1
            } else {
                2
            }
        };

        const lut: [usize; 4] = [1, 2, 0, 1];

        let idu = lut[id];
        let idv = lut[id + 1];

        // project to 2D
        let kp = Vec3::new([pos.index_unchecked(idu), pos.index_unchecked(idv), 0.0]);
        let ka = Vec3::new([a.index_unchecked(idu), a.index_unchecked(idv), 0.0]);
        let kb = Vec3::new([b.index_unchecked(idu), b.index_unchecked(idv), 0.0]);
        let kc = Vec3::new([c.index_unchecked(idu), c.index_unchecked(idv), 0.0]);

        // find barycentric coords of the quadrilateral
        let kg = kc - kb - ka;

        let k0: f32 = Vec3::cross2d_z(kp, kb);
        let k2: f32 = Vec3::cross2d_z(kc - kb, ka); // float k2 = cross2d( kg, ka );
        let k1: f32 = Vec3::cross2d_z(kp, kg) - nor.index_unchecked(id); // float k1 = cross2d( kb, ka ) + cross2d( kp, kg );

        // if edges are parallel, this is a linear equation
        let u: f32;
        let mut v: f32;

        if k2.abs() < 0.00001 {
            v = -k0 / k1;
            u = Vec3::cross2d_z(kp, ka) / k1;
        } else {
            // otherwise, it's a quadratic
            let w = k1 * k1 - 4.0 * k0 * k2;
            if w < 0.0 {
                return None;
            }
            let w = w.sqrt();

            let ik2 = 1.0 / (2.0 * k2);

            v = (-k1 - w) * ik2;
            if (v < 0.0 || v > 1.0) {
                v = (-k1 + w) * ik2;
            }

            u = (kp.x() - ka.x() * v) / (kb.x() + kg.x() * v);
        }

        if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
            return None;
        }

        // scale uv
        let u = (u * UV_SCALE).fract();
        let v = (v * UV_SCALE).fract();

        let intersection_point = pos;
        let distance_traversed = (intersection_point - ray.origin()).length();

        return Some(CastResult {
            distance_traversed,
            intersection_point,
            uv: (u, v),
            normal: Vec3::UP,
            material: self.material.clone(),
        });
    }

    fn material(&self) -> &Material {
        return self.material.get();
    }

    fn uv(&self, intersection_point: Vec3) -> (f32, f32) {
        // pee pee poo poo
        panic!("quad.uv");
        (0.0, 0.0)
    }
}
