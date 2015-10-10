use ray;
use rng::RNG;
use sampler;
use spectrum::Spectrum;
use scene;
use intersection;

pub trait Renderer {
    fn render(&mut self, &scene::Scene);

    fn li<T:RNG>(
        &self, &scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T,
        &mut Option<intersection::Intersection>,
        &mut Option<Spectrum>) -> Spectrum;

    fn transmittance<T:RNG>(
        &self, &scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T) -> Spectrum;

    // Rnderer Interface
}
