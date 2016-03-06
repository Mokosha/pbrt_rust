use sampler::base::SamplerBase;

pub struct LDSampler {
    base: SamplerBase,
    x_pos: i32,
    y_pos: i32,
    sample_buf: Vec<f32>
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
}
