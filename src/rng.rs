pub struct RNG;

impl RNG {
    pub fn new(task_idx: i32) -> RNG { RNG }
    pub fn random_float(&mut self) -> f32 {
        unimplemented!()
    }

    pub fn random_uint(&mut self) -> usize {
        unimplemented!()
    }
}

pub fn shuffle<T: Copy>(v: &mut [T], dims: usize, rng: &mut RNG) {
    let count = v.len() / dims;
    assert!(count * dims == v.len());

    for i in 0..count {
        let other = i + rng.random_uint() % (count - i);
        for j in 0..dims {
            let t = v[dims*i+j];
            v[dims*i+j] = v[dims*other+j];
            v[dims*other+j] = t;
        }
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
