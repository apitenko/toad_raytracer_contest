use rand::Rng;

use super::Vec3;

pub fn random_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let p = 2.0 * Vec3::new([rng.gen(), rng.gen(), rng.gen()]) - Vec3::ONE;
        if p.squared_length() < 1.0 {
            break p;
        }
    }
}
