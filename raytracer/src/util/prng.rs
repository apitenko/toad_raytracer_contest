use rand::{rngs::mock::StepRng, Rng};

use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

// pub struct HopefullyFasterPRNG {
//     seed: u64,
// }

// impl HopefullyFasterPRNG {
//     // Takes our seed, updates it, and returns a pseudorandom float in [0..1]
//     #[inline]
//     pub fn gen(&mut self) -> f32 {
//         self.seed = 1664525u64 * self.seed + 1013904223u64;
//         return (self.seed & 0x00FFFFFF) as f32 / 0x01000000 as f32;
//     }

//     pub fn gen_raw(&mut self) -> u64 {
//         self.seed = 1664525u64 * self.seed + 1013904223u64;
//         return self.seed;
//     }
// }

// pub struct YoloStepRng {
//     seed: u32,
// }

// impl YoloStepRng {
//     #[inline]
//     pub fn gen(&mut self) -> f32 {
//         unsafe { std::mem::transmute::<u32, f32>(self.gen_raw() & 0x3f7c7c78 | 0x38000000) }
//     }

//     #[inline]
//     pub fn gen_raw(&mut self) -> u32 {
//         self.seed = self.seed.wrapping_add(1013904223);
//         return self.seed;
//     }
// }

// #[thread_local]
// static mut LOCAL_RNG: once_cell::unsync::Lazy<HopefullyFasterPRNG> =
//     once_cell::unsync::Lazy::new(|| {
//         let mut seed = 0;
//         unsafe {
//             if core::arch::x86_64::_rdseed64_step(&mut seed) == 0 {
//                 let mut rng = rand::thread_rng();
//                 seed = rng.gen::<u64>();
//             }
//         }

//         HopefullyFasterPRNG { seed }
//     });

#[thread_local]
static mut LOCAL_RNG: once_cell::unsync::Lazy<Xoshiro256Plus> =
    once_cell::unsync::Lazy::new(|| Xoshiro256Plus::from_entropy());

// #[thread_local]
// static mut LOCAL_RNG: once_cell::unsync::Lazy<YoloStepRng> = once_cell::unsync::Lazy::new(|| {
//     let mut seed = 0;
//     unsafe {
//         if core::arch::x86_64::_rdseed64_step(&mut seed) == 0 {
//             let mut rng = rand::thread_rng();
//             seed = rng.gen::<u64>();
//         }
//     }

//     YoloStepRng { seed: seed as u32 }
// });

#[cfg(test)]
mod tests {
    use super::rand01;

    #[test]
    fn rand01_test() {
        for _ in 0..10_000 {
            let u = rand01();
            assert!(u >= 0.0);
            assert!(u <= 1.0);
        }
    }
    #[test]
    fn test_YoloStepRng() {
        let mut buckets = [0; 10];
        for _ in 0..10_000 {
            let num = rand01();

            let bucket_index = (num * 10.0).floor() as i64;
            assert!(bucket_index < 10);
            assert!(bucket_index >= 0);
            buckets[bucket_index as usize] += 1;
        }

        println!("{:?}", buckets);
    }

    // use super::YoloStepRng;

    // #[test]
    // fn test_YoloStepRng() {
    //     let mut rng = YoloStepRng { seed: 0 };
    //     let mut buckets = [0; 10];
    //     for _ in 0..10_000 {
    //         let num = rng.gen();

    //         let bucket_index = (num * 10.0).floor() as i64;
    //         assert!(bucket_index < 10);
    //         assert!(bucket_index >= 0);
    //         buckets[bucket_index as usize] += 1;
    //     }

    //     println!("{:?}", buckets);
    // }
}

#[inline]
pub fn rand01() -> f32 {
    unsafe {
        // let mut rng = rand::thread_rng();
        // return rng.gen::<f32>();
        return LOCAL_RNG.gen();
    }
}


#[inline]
pub fn rand_range(max: usize) -> usize {
    unsafe {
        // let mut rng = rand::thread_rng();
        // return rng.gen_range(0..max);
        // let a = Xoshiro256Plus::seed_from_u64(0);
        LOCAL_RNG.gen_range(0..max)
        // return (LOCAL_RNG.gen_raw() as usize % max) as usize;
    }
}
