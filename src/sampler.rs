use integrator;
use scene;

#[derive(Debug, Copy, Clone)]
pub struct Sample {
    pub idx: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Sampler;

impl Sample {
    pub fn new(sampler: &Sampler,
               surf: &integrator::SurfaceIntegrator,
               vol: &integrator::VolumeIntegrator,
               scene: &scene::Scene, idx: i32) -> Sample { Sample { idx: idx } }
}

impl Sampler {
    pub fn get_sub_sampler(&self, task_idx : i32, num_tasks : i32)
                           -> Option<Sampler> {
        Some(Sampler)
    }

    pub fn maximum_sample_count(&self) -> i32 { 1 } 
}
