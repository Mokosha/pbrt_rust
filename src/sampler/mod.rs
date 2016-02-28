mod base;
mod stratified;
pub mod sample;

use intersection::Intersection;
use ray::RayDifferential;
use rng::RNG;
use sampler::base::SamplerBase;
use sampler::sample::Sample;
use sampler::stratified::StratifiedSampler;
use spectrum::Spectrum;
use utils::Lerp;

pub enum Sampler {
    Stratified(StratifiedSampler)
}

impl Sampler {
    fn stratified(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
                  xs: usize, ys: usize, jitter: bool,
                  sopen: f32, sclose: f32) -> Sampler {
        Sampler::Stratified(StratifiedSampler::new(x_start, x_end, y_start, y_end,
                                                   xs, ys, jitter, sopen, sclose))
    }

    fn base(&self) -> &SamplerBase {
        match self {
            &Sampler::Stratified(ref sampler) => sampler.base()
        }
    }

    pub fn get_sub_sampler(&self, task_idx: usize, num_tasks: usize)
                           -> Option<Sampler> {
        match self {
            &Sampler::Stratified(ref sampler) =>
                sampler.get_sub_sampler(task_idx, num_tasks).map(Sampler::Stratified)
        }
    }

    pub fn maximum_sample_count(&self) -> usize {
        match self {
            &Sampler::Stratified(ref sampler) => sampler.maximum_sample_count()
        }
    }

    pub fn get_more_samples(&mut self, samples: &mut Vec<Sample>,
                            rng: &mut RNG) -> usize {
        match self {
            &mut Sampler::Stratified(ref mut sampler) =>
                sampler.get_more_samples(samples, rng)
        }
    }

    pub fn samples_per_pixel(&self) -> f32 {
        unimplemented!()
    }

    pub fn round_size(&self, sz: usize) -> usize {
        sz
    }

    pub fn report_results(&self, samples: &Vec<Sample>,
                          rays: &Vec<RayDifferential>,
                          ls: &Vec<Spectrum>,
                          isects: &Vec<Intersection>,
                          sample_count: usize) -> bool { true }
}
