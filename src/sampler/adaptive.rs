use sampler::base::SamplerBase;

#[derive(Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
pub enum AdaptiveTest {
    CompreShapeID,
    ContrastThreshold
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdaptiveSampler {
    base: SamplerBase,
    x_pos: i32,
    y_pos: i32,
    min_samples: usize,
    max_samples: usize,
    method: AdaptiveTest,
    supersample_pixel: bool,
    sample_buf: Vec<f32>    
}

impl AdaptiveSampler {
    pub fn new(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
               min_samples: usize, max_samples: usize, method: AdaptiveTest,
               supersample: bool, sopen: f32, sclose: f32) -> AdaptiveSampler {
        AdaptiveSampler {
            base: SamplerBase::new(x_start, x_end, y_start, y_end, max_samples,
                                   sopen, sclose),
            x_pos: x_start,
            y_pos: y_start,
            min_samples: min_samples,
            max_samples: max_samples,
            method: method,
            supersample_pixel: supersample,
            sample_buf: Vec::new()
        }
    }
    
    pub fn base(&self) -> &SamplerBase { &self.base }

    pub fn maximum_sample_count(&self) -> usize {
        self.max_samples
    }

    pub fn round_size(&self, sz: usize) -> usize {
        sz.next_power_of_two()
    }

    pub fn get_sub_sampler(&self, num: usize, count:usize) -> Option<AdaptiveSampler> {
        let (x0, x1, y0, y1) = self.base.compute_sub_window(num, count);
        if x0 == x1 || y0 == y1 {
            None
        } else {
            Some(AdaptiveSampler::new(x0, x1, y0, y1,
                                      self.min_samples, self.max_samples,
                                      self.method, self.supersample_pixel,
                                      self.base.shutter_open,
                                      self.base.shutter_close))
        }
    }
}
