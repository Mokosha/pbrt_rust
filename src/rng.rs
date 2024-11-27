extern crate rand;

use self::rand::{Rng, SeedableRng};
use self::rand::rngs::StdRng;

pub struct RNG {
    rng: StdRng
}

impl RNG {
    pub fn new(task_idx: usize) -> RNG {
        RNG { rng: rand::SeedableRng::seed_from_u64(task_idx as u64) }
    }

    pub fn random_float(&mut self) -> f32 {
        self.rng.gen::<f32>()
    }

    pub fn random_uint(&mut self) -> usize {
        (self.rng.gen::<u64>() % (usize::max_value() as u64)) as usize
    }
    
    pub fn shuffle<T>(&mut self, v: &mut [T], dims: usize) {
        let count = v.len() / dims;
        assert!(count * dims == v.len());

        for i in 0..count {
            let other = i + self.random_uint() % (count - i);
            for j in 0..dims {
                v.swap(dims*i+j, dims*other+j);
            }
        }
    }

    pub fn permutation(&mut self, v: &mut [usize]) {
        let n = v.len();
        for i in 0..n {
            v[i] = i;
        }

        self.shuffle(v, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_shuffle_slices() {
        let mut rng = RNG::new(12);
        let mut ps = [0];
        rng.shuffle(&mut ps, 1);
        assert_eq!(ps, [0]);

        let mut xs = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        rng.shuffle(&mut xs, 1);
        assert_eq!(xs[0], 1);

        let mut saw_zero = false;
        for &i in xs.iter() {
            if i == 0 {
                assert!(!saw_zero);
                saw_zero = true;
            } else {
                assert_eq!(i, 1);
            }
        }
    }

    #[test]
    fn it_can_generate_permutations() {
        let mut perm: [usize; 11] = [0; 11];
        let mut rng = RNG::new(120);
        rng.permutation(&mut perm);

        assert!(perm[0] != 0);

        let mut contents = [false; 11];
        for &i in perm.iter() {
            contents[i] = true;
        }

        for &c in contents.iter() {
            assert!(c);
        }
    }
}
