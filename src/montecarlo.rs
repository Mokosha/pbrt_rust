extern crate primal;

use rng::RNG;

use std::ops::Deref;
use std::ops::DerefMut;

pub fn radical_inverse(n: usize, b: usize) -> f64 {
    let mut v = 0.0;
    let mut num = n;
    let invBase = 1.0 / (b as f64);
    let mut aib = 1.0;
    while num > 0 {
        let d = (num % b) as f64;
        num /= b;
        aib *= invBase;
        v += d * aib;
    }

    v
}

pub fn permuted_radical_inverse(n: usize, b: usize, p: &[usize]) -> f64 {
    assert!(p.len() == b);
    let mut v = 0.0;
    let mut num = n;
    let invBase = 1.0 / (b as f64);
    let mut aib = 1.0;
    while num > 0 {
        let d = p[num % b] as f64;
        num /= b;
        aib *= invBase;
        v += d * aib;
    }

    v
}

pub struct PermutedHalton {
    dims: usize,
    bases: Vec<usize>,
    permute: Vec<usize>
}

impl PermutedHalton {
    pub fn new(d: usize, rng: &mut RNG) -> PermutedHalton {

        // Determine bases and their sum
        let b: Vec<usize> = primal::Primes::all().take(d).collect();
        let sum_bases = b.iter().fold(0, |acc, &x| acc + x);

        // Compute permutation tables for each base
        let mut perms = vec![0; sum_bases];
        b.iter().fold((perms.deref_mut(), 0), |(xs, off), &x| {
            let (mut p, mut rest) = xs.split_at_mut(x);
            rng.permutation(p);
            (rest, off + x)
        });

        PermutedHalton {
            dims: d,
            bases: b,
            permute: perms
        }
    }

    pub fn sample(&self, n: usize, out: &mut [f32]) {
        assert!(out.len() == self.bases.len());
        self.bases.iter().enumerate().fold((self.permute.deref(), 0), |(xs, off), (i, &x)| {
            let (p, rest) = xs.split_at(x);
            out[i] = permuted_radical_inverse(n, x, p) as f32;
            (rest, off + x)
        });
    }
}

pub fn latin_hypercube(samples: &mut [f32], num: usize, dim: usize, rng: &mut RNG) {
    // Generate LHS samples along diagonal
    let delta = 1.0 / (num as f32);
    for i in 0..num {
        for j in 0..dim {
            samples[dim * i + j] = ((i as f32) + rng.random_float()) * delta;
        }
    }

    // Permute LHS samples in each dimension
    for i in 0..dim {
        for j in 0..num {
            let other = j + rng.random_uint() % (num - j);
            samples.swap(dim*j + i, dim*other + i);
        }
    }
}

pub fn stratified_sample_1d(samples: &mut [f32], num_samples: usize,
                            rng: &mut RNG, jitter: bool) {
    let inv_tot = 1.0 / (num_samples as f32);
    for i in 0..num_samples {
        let delta = if jitter { rng.random_float() } else { 0.5 };
        samples[i] = ((i as f32) + delta) * inv_tot;
    }
}

pub fn stratified_sample_2d(samples: &mut [f32], nx: usize, ny: usize,
                            rng: &mut RNG, jitter: bool) {
    let dx = 1.0 / (nx as f32);
    let dy = 1.0 / (ny as f32);
    for y in 0..ny {
        for x in 0..nx {
            let jx = if jitter { rng.random_float() } else { 0.5 };
            let jy = if jitter { rng.random_float() } else { 0.5 };
            let off = 2 * (y*nx + x);
            samples[off] = ((x as f32) + jx) * dx;
            samples[off + 1] = ((y as f32) + jy) * dy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rng::RNG;

    #[test]
    fn it_can_compute_radical_inverses() {
        assert_eq!(radical_inverse(0, 2), 0.0);
        assert_eq!(radical_inverse(1, 2), 0.5);
        assert_eq!(radical_inverse(2, 2), 0.25);
        assert_eq!(radical_inverse(3, 2), 0.75);
        assert_eq!(radical_inverse(4, 2), 0.125);
        assert_eq!(radical_inverse(5, 2), 0.625);
        assert_eq!(radical_inverse(6, 2), 0.375);
        assert_eq!(radical_inverse(7, 2), 0.875);

        assert_eq!(radical_inverse(0, 4), 0.0);
        assert_eq!(radical_inverse(1, 4), 0.25);
        assert_eq!(radical_inverse(2, 4), 0.5);
        assert_eq!(radical_inverse(3, 4), 0.75);
        assert_eq!(radical_inverse(4, 4), 1.0 / 16.0);
        assert_eq!(radical_inverse(5, 4), 5.0 / 16.0);
        assert_eq!(radical_inverse(6, 4), 9.0 / 16.0);
        assert_eq!(radical_inverse(7, 4), 13.0 / 16.0);
        assert_eq!(radical_inverse(8, 4), 2.0 / 16.0);
        assert_eq!(radical_inverse(9, 4), 6.0 / 16.0);
        assert_eq!(radical_inverse(10, 4), 10.0 / 16.0);
        assert_eq!(radical_inverse(11, 4), 14.0 / 16.0);

        assert!((radical_inverse(0, 3) - (0.0)).abs() < 1e-6);
        assert!((radical_inverse(1, 3) - (1.0 / 3.0)).abs() < 1e-6);
        assert!((radical_inverse(2, 3) - (2.0 / 3.0)).abs() < 1e-6);
        assert!((radical_inverse(3, 3) - (1.0 / 9.0)).abs() < 1e-6);
        assert!((radical_inverse(4, 3) - (4.0 / 9.0)).abs() < 1e-6);
        assert!((radical_inverse(5, 3) - (7.0 / 9.0)).abs() < 1e-6);
        assert!((radical_inverse(6, 3) - (2.0 / 9.0)).abs() < 1e-6);
        assert!((radical_inverse(7, 3) - (5.0 / 9.0)).abs() < 1e-6);
        assert!((radical_inverse(8, 3) - (8.0 / 9.0)).abs() < 1e-6);
    }

    #[test]
    fn it_can_stratify_1d() {
        let mut fs = [0.0; 4];
        let mut rng = RNG::new(0);
        stratified_sample_1d(&mut fs, 4, &mut rng, false);

        assert_eq!(fs[0], 0.5 / 4.0);
        assert_eq!(fs[1], 1.5 / 4.0);
        assert_eq!(fs[2], 2.5 / 4.0);
        assert_eq!(fs[3], 3.5 / 4.0);

        stratified_sample_1d(&mut fs, 4, &mut rng, true);

        assert!(0.0 / 4.0 <= fs[0] && fs[0] <= 1.0 / 4.0);
        assert!(1.0 / 4.0 <= fs[1] && fs[1] <= 2.0 / 4.0);
        assert!(2.0 / 4.0 <= fs[2] && fs[2] <= 3.0 / 4.0);
        assert!(3.0 / 4.0 <= fs[3] && fs[3] <= 4.0 / 4.0);
    }

    #[test]
    fn it_can_stratify_2d() {
        let mut fs = [0.0; 12];
        let mut rng = RNG::new(0);
        stratified_sample_2d(&mut fs, 2, 2, &mut rng, false);

        assert_eq!(fs[0], 0.5 / 2.0);
        assert_eq!(fs[1], 0.5 / 2.0);

        assert_eq!(fs[2], 1.5 / 2.0);
        assert_eq!(fs[3], 0.5 / 2.0);

        assert_eq!(fs[4], 0.5 / 2.0);
        assert_eq!(fs[5], 1.5 / 2.0);

        assert_eq!(fs[6], 1.5 / 2.0);
        assert_eq!(fs[7], 1.5 / 2.0);

        stratified_sample_2d(&mut fs, 3, 2, &mut rng, true);

        assert!(0.0 / 3.0 <= fs[0] && fs[0] <= 1.0 / 3.0);
        assert!(0.0 / 2.0 <= fs[1] && fs[1] <= 1.0 / 2.0);

        assert!(1.0 / 3.0 <= fs[2] && fs[2] <= 2.0 / 3.0);
        assert!(0.0 / 2.0 <= fs[3] && fs[3] <= 1.0 / 2.0);

        assert!(2.0 / 3.0 <= fs[4] && fs[4] <= 3.0 / 3.0);
        assert!(0.0 / 2.0 <= fs[5] && fs[5] <= 1.0 / 2.0);

        assert!(0.0 / 3.0 <= fs[6] && fs[6] <= 1.0 / 3.0);
        assert!(1.0 / 2.0 <= fs[7] && fs[7] <= 2.0 / 2.0);

        assert!(1.0 / 3.0 <= fs[8] && fs[8] <= 2.0 / 3.0);
        assert!(1.0 / 2.0 <= fs[9] && fs[9] <= 2.0 / 2.0);

        assert!(2.0 / 3.0 <= fs[10] && fs[10] <= 3.0 / 3.0);
        assert!(1.0 / 2.0 <= fs[11] && fs[11] <= 2.0 / 2.0);
    }

    #[test]
    fn it_can_generate_permuted_halton_samples() {
        let mut rng = RNG::new(42);
        let halton = PermutedHalton::new(3, &mut rng);

        let mut sample = [
            [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]];

        halton.sample(1, &mut sample[0]);
        halton.sample(2, &mut sample[1]);
        halton.sample(3, &mut sample[2]);
        halton.sample(4, &mut sample[3]);

        let mut is_there = [false; 4];

        let dim2 = [0.5, 0.25, 0.75, 0.125];
        for (i, &d) in dim2.iter().enumerate() {
            for s in sample.iter() {
                if s[0] == d {
                    is_there[i] = true;
                }
            }
        }

        for t in is_there.iter_mut() {
            assert!(*t);
            *t = false;
        }

        let dim3 = [0.0, 1.0 / 3.0, 2.0 / 3.0, 0.0];
        for (i, &d) in dim3.iter().enumerate() {
            for s in sample.iter() {
                if s[1] == d {
                    is_there[i] = true;
                }
            }
        }

        for t in is_there.iter_mut() {
            assert!(*t);
            *t = false;
        }

        let dim5 = [0.0, 4.0 / 5.0, 2.0 / 5.0, 3.0 / 5.0];
        for &d in dim5.iter() {
            for (i, s) in sample.iter().enumerate() {
                if s[2] == d {
                    is_there[i] = true;
                }
            }
        }

        for t in is_there.iter_mut() {
            assert!(*t);
            *t = false;
        }
    }

    #[ignore]
    #[test]
    fn it_can_generate_latin_hypercube() {
    }
}
