use crate::util::prng::rand01;

use super::vec3::Vec3;

pub fn random_in_unit_sphere() -> Vec3 {
    loop {
        let p = 2.0 * Vec3::new([rand01(), rand01(), rand01()]) - Vec3::ONE;
        if p.squared_length() < 1.0 {
            break p;
        }
    }
}
