use ray;
use rng::RNG;
use sampler;
use spectrum::Spectrum;
use scene;
use intersection::Intersection;

pub trait Renderer {
    fn render(&mut self, &scene::Scene);

    fn li<'a, T:RNG>(
        &self, &'a scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T) -> (Spectrum, Option<Intersection<'a>>, Spectrum);

    fn li_simple<T:RNG>(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &sampler::Sample, rng: &mut T) -> Spectrum {
        self.li(scene, ray, sample, rng).0
    }

    fn transmittance<T:RNG>(
        &self, &scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T) -> Spectrum;

    // Rnderer Interface
}
