use ray;
use sampler;
use scene;
use intersection;

pub struct Spectrum;

pub trait RNG {
}

pub trait Renderer {
    fn render(&mut self, &scene::Scene);

    fn li<T:RNG>(
        &self, &scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T,
        &mut Option<intersection::Intersection>,
        &mut Option<Spectrum>);

    fn transmittance<T:RNG>(
        &self, &scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T);

    // Rnderer Interface
}
