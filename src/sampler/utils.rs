use camera::CameraSample;
use rng::RNG;
use sampler::sample::Sample;
use utils::Lerp;

pub fn van_der_corput(_n: u32, scramble: u32) -> f32 {
    let mut n = _n;

    // Reverse bits of n
    n = (n << 16) | (n >> 16);
    n = ((n & 0x00ff00ff) << 8) | ((n & 0xff00ff00) >> 8);
    n = ((n & 0x0f0f0f0f) << 4) | ((n & 0xf0f0f0f0) >> 4);
    n = ((n & 0x33333333) << 2) | ((n & 0xCCCCCCCC) >> 2);
    n = ((n & 0x55555555) << 2) | ((n & 0xAAAAAAAA) >> 2);
    
    n ^= scramble;
    ((((n >> 8) & 0xffffff) as f64) / ((1 << 24) as f64)) as f32
}

fn sobol2(_n: u32, scramble: u32) -> f32 {
    let mut s = scramble;
    let mut n = _n;
    let mut v: u32 = 1 << 31;
    while n != 0 {
        if (n & 0x1) == 0 {
            s ^= v;
        }
        v ^= v >> 1;
        n >>= 1;
    }

    ((((s >> 8) & 0xFFFFFF) as f64) / ((1 << 24) as f64)) as f32
}

pub fn sample02(n: u32, scramble: [u32; 2]) -> (f32, f32) {
    (van_der_corput(n, scramble[0]), sobol2(n, scramble[1]))
}

pub fn ld_pixel_sample_floats_needed(sample: &Sample,
                                     num_pixel_samples: usize) -> usize {
    let mut n = 5;  // 2 image, 2 lens, 1 time
    n += sample.num_1d.iter().fold(0, |acc, &x| acc + x);
    n += sample.num_2d.iter().fold(0, |acc, &x| acc + (2 * x));
    n * num_pixel_samples
}

fn ld_shuffle_scrambled_1d(num_samples: usize, num_pixel_samples: usize,
                               samples: &mut [f32], rng: &mut RNG) {
    assert!(samples.len() >= num_samples * num_pixel_samples);

    let scramble = rng.random_uint();
    for i in 0..(num_samples * num_pixel_samples) {
        samples[i] = van_der_corput(i as u32, scramble as u32);
    }

    for win in samples.chunks_mut(num_samples) {
        debug_assert_eq!(win.len(), num_samples);
        rng.shuffle(win, 1);
    }

    rng.shuffle(samples, num_samples);
}

fn ld_shuffle_scrambled_2d(num_samples: usize, num_pixel_samples: usize,
                           samples: &mut [f32], rng: &mut RNG) {
    assert!(samples.len() >= num_samples * num_pixel_samples * 2);

    let scramble = [rng.random_uint() as u32, rng.random_uint() as u32];
    for i in 0..(num_samples * num_pixel_samples) {
        let (s1, s2) = sample02(i as u32, scramble);
        samples[2 * i] = s1;
        samples[2 * i + 1] = s2;
    }

    for win in samples.chunks_mut(2 * num_samples) {
        debug_assert_eq!(win.len(), num_samples);
        rng.shuffle(win, 2);
    }

    rng.shuffle(samples, 2 * num_samples);
}

pub fn ld_pixel_sample(x_pos: i32, y_pos: i32, shutter_open: f32, shutter_close: f32,
                       num_samples: usize, samples: &mut [Sample],
                       buf: &mut [f32], rng: &mut RNG) {
    if samples.is_empty() {
        return;
    }

    // Prepare temporary array pointers for low-discrepancy camera samples
    let (mut image_samples, mut not_image_samples) =
        buf.split_at_mut(2 * num_samples);
    let (mut lens_samples, mut not_lens_samples) =
        not_image_samples.split_at_mut(2 * num_samples);
    let (mut time_samples, mut not_time_samples) =
        not_lens_samples.split_at_mut(num_samples);

    // Prepare temporary array pointers for low-discrepancy integrator samples
    let total_oned_samples =
        samples[0].num_1d.iter()
        .fold(0, |acc, &x| acc + (x * num_samples));
    let (oned_sample_buf, twod_sample_buf) =
        not_time_samples.split_at_mut(total_oned_samples);
    assert_eq!(twod_sample_buf.len(),
               samples[0].num_2d.iter().fold(0, |acc, &x| acc + (2 * x * num_samples)));

    // !SPEED! These are allocated on the heap. :(
    let mut oned_samples = samples[0].num_1d.iter()
        .fold((Vec::new(), oned_sample_buf), |(mut ss, rest), &split| {
            let (oned, the_rest) = rest.split_at_mut(split);
            ss.push(oned);
            (ss, the_rest)
        }).0;

    let mut twod_samples = samples[0].num_2d.iter()
        .fold((Vec::new(), twod_sample_buf), |(mut ss, rest), &split| {
            let (twod, the_rest) = rest.split_at_mut(2 * split);
            ss.push(twod);
            (ss, the_rest)
        }).0;

    // Generate low-discrepancy pixel samples
    ld_shuffle_scrambled_2d(1, num_samples, &mut image_samples, rng);
    ld_shuffle_scrambled_2d(1, num_samples, &mut lens_samples, rng);
    ld_shuffle_scrambled_1d(1, num_samples, &mut time_samples, rng);

    for (i, oned) in oned_samples.iter_mut().enumerate() {
        ld_shuffle_scrambled_1d(samples[0].num_1d[i], num_samples, oned, rng);
    }

    for (i, twod) in twod_samples.iter_mut().enumerate() {
        ld_shuffle_scrambled_2d(samples[0].num_2d[i], num_samples, twod, rng);
    }

    // Initialize samples with computed sample values
    for i in 0..num_samples {
        let t = shutter_open.lerp(&shutter_close, time_samples[i]);
        samples[i].camera_sample =
            CameraSample::new(x_pos as f32 + image_samples[2 * i],
                              y_pos as f32 + image_samples[2 * i + 1],
                              lens_samples[2 * i],
                              lens_samples[2 * i + 1],
                              t);

        // Copy integrator samples into samples[i]
        // !KLUDGE! This isn't very rust-y
        for j in 0..(samples[i].num_1d.len()) {
            let start_samp = samples[i].num_1d[j] * i;
            for k in 0..(samples[i].num_1d[j]) {
                samples[i].samples[samples[i].offset_1d[j] + k] =
                    oned_samples[j][start_samp + k];
            }
        }

        for j in 0..(samples[i].num_2d.len()) {
            let start_samp = 2 * samples[i].num_2d[j] * i;
            for k in 0..(2 * samples[i].num_2d[j]) {
                samples[i].samples[samples[i].offset_2d[j] + k] =
                    twod_samples[j][start_samp + k];
            }
        }
    }
}
