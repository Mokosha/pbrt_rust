extern crate rand;

use self::rand::{Rng, SeedableRng, XorShiftRng};

pub struct RNG {
    rng: XorShiftRng
}

impl RNG {
    pub fn new(task_idx: usize) -> RNG {
        let s = task_idx + 1;
        let seed = [
            s as u32,
            (s * s) as u32,
            (s * s * s) as u32,
            (s * s * s * s) as u32];

        RNG {
            rng: SeedableRng::from_seed(seed)
        }
    }

    pub fn random_float(&mut self) -> f32 {
        self.rng.next_f32()
    }

    pub fn random_uint(&mut self) -> usize {
        (self.rng.next_u64() % (usize::max_value() as u64)) as usize
    }
}

impl RNG {
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

    #[ignore]
    #[test]
    fn it_can_shuffle_slices() {
        unimplemented!()
    }
}
