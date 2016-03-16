use intersection::Intersection;
use ray::RayDifferential;
use rng::RNG;
use sampler::sample::Sample;
use sampler::base::SamplerBase;
use spectrum::Spectrum;

use sampler::utils::*;

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

    pub fn get_more_samples(&mut self, samples: &mut Vec<Sample>,
                            rng: &mut RNG) -> usize {
        assert!(samples.len() >= 1);
        if self.y_pos == self.base.y_pixel_end { return 0 }

        let ns = ld_pixel_sample_floats_needed(samples.get(0).unwrap(),
                                               self.base.samples_per_pixel);
        self.sample_buf.resize(ns, 0f32);

        let num_samples =
            if self.supersample_pixel {
                self.max_samples
            } else {
                self.min_samples
            };

        ld_pixel_sample(self.x_pos, self.y_pos, self.base.shutter_open,
                        self.base.shutter_close, num_samples, samples,
                        &mut self.sample_buf, rng);

        self.supersample_pixel = false;

        num_samples
    }

    pub fn report_results(&mut self, samples: &Vec<Sample>,
                          rays: &Vec<RayDifferential>,
                          ls: &Vec<Spectrum>, isects: &Vec<Intersection>,
                          sample_count: usize) -> bool {
        let needs_supersample = !self.supersample_pixel && match self.method {
            AdaptiveTest::CompreShapeID => {
                let tail = isects.iter().skip(1);
                isects.iter().zip(tail).fold(false, |acc, (i1, i2)| {
                    acc
                        || (i1.shape_id != i2.shape_id)
                        || (i1.primitive_id != i2.primitive_id)
                })
            },

            AdaptiveTest::ContrastThreshold => {
                let lavg = ls.iter().fold(0.0, |acc, l| {
                    acc + l.y()
                });

                let contrast_ratio = 0.5;
                ls.iter().fold(false, |acc, l| {
                    acc || ((l.y() - lavg).abs() / lavg) > contrast_ratio
                })
            }
        };

        if needs_supersample {
            self.supersample_pixel = true;
            return false;
        }

        self.x_pos += 1;
        if self.x_pos == self.base.x_pixel_end {
            self.x_pos = self.base.x_pixel_start;
            self.y_pos += 1;
        }

        true
    }
}
