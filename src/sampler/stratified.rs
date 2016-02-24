use sampler::SamplerBase;

pub struct StratifiedSampler {
    base: SamplerBase,
    x_pixel_samples: usize,
    y_pixel_samples: usize,
    jitter_samples: bool,
    x_pos: i32,
    y_pos: i32,
    sample_buf: Vec<f32>
}

impl StratifiedSampler {
    pub fn new(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
               xs: usize, ys: usize, jitter: bool,
               sopen: f32, sclose: f32) -> StratifiedSampler {
        let b = SamplerBase::new(x_start, x_end, y_start, y_end, xs * ys, sopen, sclose);
        StratifiedSampler {
            base: b.clone(),
            x_pixel_samples: xs,
            y_pixel_samples: ys,
            jitter_samples: jitter,
            x_pos: b.x_pixel_start,
            y_pos: b.y_pixel_start,
            sample_buf: vec![0.0; 5 * (xs * ys as usize)]
        }
    }

    pub fn get_sub_sampler(&self, num: usize, count: usize) -> Option<StratifiedSampler> {
        let (x0, x1, y0, y1) = self.base.compute_sub_window(num, count);
        if x0 == x1 || y0 == y1 {
            None
        } else {
            Some(StratifiedSampler::new(x0, x1, y0, y1,
                                        self.x_pixel_samples, self.y_pixel_samples,
                                        self.jitter_samples,
                                        self.base.shutter_open, self.base.shutter_close))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_be_created() {
        unimplemented!()
    }
}
