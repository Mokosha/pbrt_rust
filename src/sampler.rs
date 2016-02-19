use camera::CameraSample;
use integrator::SurfaceIntegrator;
use integrator::VolumeIntegrator;
use intersection::Intersection;
use ray::RayDifferential;
use rng::RNG;
use spectrum::Spectrum;
use scene;

#[derive(Debug, Clone)]
pub struct Sample {
    camera_sample: CameraSample,
}

#[derive(Debug, Copy, Clone)]
pub struct Sampler;

impl Sample {
    pub fn new<Surf : SurfaceIntegrator, Vol : VolumeIntegrator>(
        sampler: &Sampler, surf: &Surf, vol: &Vol, scene: &scene::Scene, idx: i32)
        -> Sample { unimplemented!() }

    pub fn to_camera_sample(self) -> CameraSample { self.camera_sample }
}

impl Sampler {
    pub fn get_sub_sampler(&self, task_idx : i32, num_tasks : i32)
                           -> Option<Sampler> {
        Some(Sampler)
    }

    pub fn maximum_sample_count(&self) -> i32 { 1 }

    pub fn get_more_samples<T: RNG>(&mut self, samples: &mut Vec<Sample>, rng: &mut T) {

    }

    pub fn samples_per_pixel(&self) -> f32 { 0.0f32 }

    pub fn report_results(&self, samples: &Vec<Sample>,
                          rays: &Vec<RayDifferential>,
                          ls: &Vec<Spectrum>,
                          isects: &Vec<Intersection>,
                          sample_count: usize) -> bool { true }
}
