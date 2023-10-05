use crate::primitives::bounding_box::BoundingBox;

use super::Vec3;

pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
}

pub fn sphere_bbox_intersection(sphere: &Sphere, bbox: &BoundingBox) -> bool {
    #[inline]
    fn squared(i: f32) -> f32 {
        i * i
    }

    let mut dist_squared = sphere.radius * sphere.radius;

    let S = sphere.position;
    let C1 = bbox.min;
    let C2 = bbox.max;
    /* assume C1 and C2 are element-wise sorted, if not, do that now */

    if S.x() < C1.x() {
        dist_squared -= squared(S.x() - C1.x());
    } else if S.x() > C2.x() {
        dist_squared -= squared(S.x() - C2.x());
    }
    if S.y() < C1.y() {
        dist_squared -= squared(S.y() - C1.y());
    } else if S.y() > C2.y() {
        dist_squared -= squared(S.y() - C2.y());
    }
    if S.z() < C1.z() {
        dist_squared -= squared(S.z() - C1.z());
    } else if S.z() > C2.z() {
        dist_squared -= squared(S.z() - C2.z());
    }
    return dist_squared > 0.0;
}
