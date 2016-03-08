use camera::CameraSample;
use rng::RNG;
use sampler::sample::Sample;
use sampler::base::SamplerBase;
use utils::Lerp;

use sampler::utils::van_der_corput;
use sampler::utils::sample02;

pub struct LDSampler {
    base: SamplerBase,
    x_pos: i32,
    y_pos: i32,
    sample_buf: Vec<f32>
}

fn ld_pixel_sample_floats_needed(sample: &Sample, num_pixel_samples: usize) -> usize {
    let mut n = 5;  // 2 image, 2 lens, 1 time
    n += sample.n1D.iter().fold(0, |acc, &x| acc + x);
    n += sample.n2D.iter().fold(0, |acc, &x| acc + (2 * x));
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

fn ld_pixel_sample(x_pos: i32, y_pos: i32, shutter_open: f32, shutter_close: f32,
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
    let total_oneD_samples = samples[0].n1D.iter().fold(0, |acc, &x| acc + (x * num_samples));
    let (mut oneD_sample_buf, mut twoD_sample_buf) =
        not_time_samples.split_at_mut(total_oneD_samples);
    assert_eq!(twoD_sample_buf.len(),
               samples[0].n2D.iter().fold(0, |acc, &x| acc + (2 * x * num_samples)));

    // !SPEED! These are allocated on the heap. :(
    let mut oneD_samples = samples[0].n1D.iter()
        .fold((Vec::new(), oneD_sample_buf), |(mut ss, rest), &split| {
            let (mut oneD, mut the_rest) = rest.split_at_mut(split);
            ss.push(oneD);
            (ss, the_rest)
        }).0;

    let mut twoD_samples = samples[0].n2D.iter()
        .fold((Vec::new(), twoD_sample_buf), |(mut ss, rest), &split| {
            let (mut twoD, mut the_rest) = rest.split_at_mut(2 * split);
            ss.push(twoD);
            (ss, the_rest)
        }).0;

    // Generate low-discrepancy pixel samples
    ld_shuffle_scrambled_2d(1, num_samples, &mut image_samples, rng);
    ld_shuffle_scrambled_2d(1, num_samples, &mut lens_samples, rng);
    ld_shuffle_scrambled_1d(1, num_samples, &mut time_samples, rng);

    for (i, oneD) in oneD_samples.iter_mut().enumerate() {
        ld_shuffle_scrambled_1d(samples[0].n1D[i], num_samples, oneD, rng);
    }

    for (i, twoD) in twoD_samples.iter_mut().enumerate() {
        ld_shuffle_scrambled_2d(samples[0].n2D[i], num_samples, twoD, rng);
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
        for j in 0..(samples[i].n1D.len()) {
            let start_samp = samples[i].n1D[j] * i;
            for k in 0..(samples[i].n1D[j]) {
                samples[i].samples[offset1D[j] + k] = oneD_samples[j][start_samp + k];
            }
        }

        for j in 0..(samples[i].n2D.len()) {
            let start_samp = 2 * samples[i].n2D[j] * i;
            for k in 0..(2 * samples[i].n2D[j]) {
                samples[i].samples[offset2D[j] + k] = twoD_samples[j][start_samp + k];
            }
        }
    }
}

impl LDSampler {
    pub fn new(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
               samples_per_pixel: usize, sopen: f32, sclose: f32) -> LDSampler {
        let ps = samples_per_pixel.next_power_of_two();
        if !samples_per_pixel.is_power_of_two() {
            print!("Warning -- ");
            println!("LDSampler using next power of two ({:?}) samples per pixel", ps);
        }

        LDSampler {
            base: SamplerBase::new(x_start, x_end, y_start, y_end, ps, sopen, sclose),
            x_pos: x_start,
            y_pos: y_start,
            sample_buf: Vec::new()
        }
    }

    pub fn base(&self) -> &SamplerBase { &self.base }

    pub fn maximum_sample_count(&self) -> usize {
        self.base.samples_per_pixel
    }

    pub fn round_size(&self, sz: usize) -> usize {
        sz.next_power_of_two()
    }

    pub fn get_sub_sampler(&self, num: usize, count:usize) -> Option<LDSampler> {
        let (x0, x1, y0, y1) = self.base.compute_sub_window(num, count);
        if x0 == x1 || y0 == y1 {
            None
        } else {
            Some(LDSampler::new(x0, x1, y0, y1,
                                self.base.samples_per_pixel,
                                self.base.shutter_open,
                                self.base.shutter_close))
        }
    }

    pub fn get_more_samples(&mut self, samples: &mut Vec<Sample>,
                            rng: &mut RNG) -> usize {
        assert!(samples.len() >= 1);
        if self.y_pos == self.base.y_pixel_end { return 0 }

        let ns = ld_pixel_sample_floats_needed(samples.get(0).unwrap(),
                                               self.base.samples_per_pixel);
        self.sample_buf.resize(ns, 0f32);

        ld_pixel_sample(self.x_pos, self.y_pos, self.base.shutter_open,
                        self.base.shutter_close, self.base.samples_per_pixel,
                        samples, &mut self.sample_buf, rng);

        self.x_pos += 1;
        if self.x_pos == self.base.x_pixel_end {
            self.x_pos = self.base.x_pixel_start;
            self.y_pos += 1;
        }

        self.base.samples_per_pixel
    }
}
