use rng::RNG;
use sampler::sample::Sample;
use sampler::base::SamplerBase;

pub struct LDSampler {
    base: SamplerBase,
    x_pos: i32,
    y_pos: i32,
    sample_buf: Vec<f32>
}

fn ld_pixel_sample_floats_needed(sample: &Sample, num_pixel_samples: usize) -> usize {
    let mut n = 5;  // 2 image, 2 lens, 1 time
    for &ns in sample.n1D.iter() {
        n += ns;
    }

    for &ns in sample.n2D.iter() {
        n += 2 * ns;
    }

    n * num_pixel_samples
}

fn ld_pixel_sample(x_pos: i32, y_pos: i32, shutter_open: f32, shutter_close: f32,
                   num_pixel_samples: usize, sample: &mut Vec<Sample>,
                   buf: &mut [f32], rng: &mut RNG) {
    // Prepare temporary array pointers for low-discrepancy camera samples
    // Prepare temporary array pointers for low-discrepancy integrator samples
    // Generate low-discrepancy pixel samples
    // Initialize samples with computed sample values
    unimplemented!()
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
