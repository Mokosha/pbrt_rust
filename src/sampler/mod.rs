mod adaptive;
mod base;
mod halton;
mod lds;
pub mod sample;
mod stratified;
mod utils;

use intersection::Intersection;
use ray::RayDifferential;
use rng::RNG;
use sampler::base::SamplerBase;
use sampler::adaptive::AdaptiveTest;
use sampler::adaptive::AdaptiveSampler;
use sampler::halton::HaltonSampler;
use sampler::lds::LDSampler;
use sampler::sample::Sample;
use sampler::stratified::StratifiedSampler;
use spectrum::Spectrum;

#[derive(Clone, Debug, PartialEq)]
pub enum Sampler {
    Stratified(StratifiedSampler),
    Halton(HaltonSampler),
    LowDiscrepancy(LDSampler),
    Adaptive(AdaptiveSampler)
}

impl Sampler {
    pub fn stratified(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
                      xs: usize, ys: usize, jitter: bool,
                      sopen: f32, sclose: f32) -> Sampler {
        Sampler::Stratified(StratifiedSampler::new(x_start, x_end, y_start, y_end,
                                                   xs, ys, jitter, sopen, sclose))
    }

    pub fn halton(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
                  samples_per_pixel: usize, sopen: f32, sclose: f32) -> Sampler {
        Sampler::Halton(HaltonSampler::new(x_start, x_end, y_start, y_end,
                                           samples_per_pixel, sopen, sclose))
    }

    pub fn low_discrepancy(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
                           samples_per_pixel: usize, sopen: f32,
                           sclose: f32) -> Sampler {
        Sampler::LowDiscrepancy(LDSampler::new(x_start, x_end, y_start, y_end,
                                               samples_per_pixel, sopen, sclose))
    }

    pub fn adaptive(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
                    min_samples: usize, max_samples: usize, method: AdaptiveTest,
                    supersample: bool, sopen: f32, sclose: f32) -> Sampler {
        Sampler::Adaptive(AdaptiveSampler::new(x_start, x_end, y_start, y_end,
                                               min_samples, max_samples, method,
                                               supersample, sopen, sclose))
    }

    fn base(&self) -> &SamplerBase {
        match self {
            &Sampler::Stratified(ref sampler) => sampler.base(),
            &Sampler::Halton(ref sampler) => sampler.base(),
            &Sampler::LowDiscrepancy(ref sampler) => sampler.base(),
            &Sampler::Adaptive(ref sampler) => sampler.base()
        }
    }

    pub fn get_sub_sampler(&self, task_idx: usize, num_tasks: usize)
                           -> Option<Sampler> {
        match self {
            &Sampler::Stratified(ref sampler) =>
                sampler
                .get_sub_sampler(task_idx, num_tasks)
                .map(Sampler::Stratified),
            &Sampler::Halton(ref sampler) =>
                sampler
                .get_sub_sampler(task_idx, num_tasks)
                .map(Sampler::Halton),
            &Sampler::LowDiscrepancy(ref sampler) =>
                sampler
                .get_sub_sampler(task_idx, num_tasks)
                .map(Sampler::LowDiscrepancy),
            &Sampler::Adaptive(ref sampler) =>
                sampler
                .get_sub_sampler(task_idx, num_tasks)
                .map(Sampler::Adaptive),
        }
    }

    pub fn maximum_sample_count(&self) -> usize {
        match self {
            &Sampler::Stratified(ref sampler) => sampler.maximum_sample_count(),
            &Sampler::Halton(ref sampler) => sampler.maximum_sample_count(),
            &Sampler::LowDiscrepancy(ref sampler) => sampler.maximum_sample_count(),
            &Sampler::Adaptive(ref sampler) => sampler.maximum_sample_count()
        }
    }

    pub fn get_more_samples(&mut self, samples: &mut Vec<Sample>,
                            rng: &mut RNG) -> usize {
        match self {
            &mut Sampler::Stratified(ref mut sampler) =>
                sampler.get_more_samples(samples, rng),
            &mut Sampler::Halton(ref mut sampler) =>
                sampler.get_more_samples(samples, rng),
            &mut Sampler::LowDiscrepancy(ref mut sampler) =>
                sampler.get_more_samples(samples, rng),
            &mut Sampler::Adaptive(ref mut sampler) =>
                sampler.get_more_samples(samples, rng)
        }
    }

    pub fn samples_per_pixel(&self) -> f32 {
        self.base().samples_per_pixel as f32
    }

    pub fn round_size(&self, sz: usize) -> usize {
        match self {
            &Sampler::LowDiscrepancy(_) => sz.next_power_of_two(),
            &Sampler::Adaptive(_) => sz.next_power_of_two(),
            _ => sz
        }
    }

    pub fn report_results(&mut self, samples: &Vec<Sample>,
                          rays: &Vec<RayDifferential>,
                          ls: &Vec<Spectrum>,
                          isects: &Vec<Intersection>,
                          sample_count: usize) -> bool {
        match self {
            &mut Sampler::Adaptive(ref mut sampler) =>
                sampler.report_results(samples, rays, ls, isects, sample_count),
            _ => true
        }
    }
}
