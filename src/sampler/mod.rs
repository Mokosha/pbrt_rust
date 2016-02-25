mod base;
mod stratified;
pub mod sample;

use intersection::Intersection;
use ray::RayDifferential;
use rng::RNG;
use sampler::sample::Sample;
use spectrum::Spectrum;
use sampler::base::SamplerBase;
use utils::Lerp;

pub struct Sampler {
    base: SamplerBase
}

impl Sampler {
    pub fn new() -> Sampler { unimplemented!() }

    fn base(&self) -> &SamplerBase { &self.base }

    pub fn get_sub_sampler(&self, task_idx : i32, num_tasks : i32)
                           -> Option<Sampler> {
        unimplemented!()
    }

    pub fn maximum_sample_count(&self) -> i32 {
        unimplemented!()
    }

    pub fn get_more_samples<T: RNG>(&mut self, samples: &mut Vec<Sample>, rng: &mut T) {
        unimplemented!()
    }

    pub fn samples_per_pixel(&self) -> f32 {
        unimplemented!()
    }

    pub fn round_size(&self, sz: usize) -> usize {
        unimplemented!()
    }

    pub fn report_results(&self, samples: &Vec<Sample>,
                          rays: &Vec<RayDifferential>,
                          ls: &Vec<Spectrum>,
                          isects: &Vec<Intersection>,
                          sample_count: usize) -> bool { true }
}
