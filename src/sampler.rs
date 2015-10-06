use integrator;
use scene;

pub struct Sample;
pub struct Sampler;

impl Sample {
    pub fn new(sampler: &Sampler,
               surf: &integrator::SurfaceIntegrator,
               vol: &integrator::VolumeIntegrator,
               scene: &scene::Scene) -> Sample { Sample }
}
