use ray;
use rng::RNG;
use sampler::sample::Sample;
use spectrum::Spectrum;
use scene;
use intersection::Intersection;

pub trait Renderer {
    fn render(&mut self, &scene::Scene);

    fn li<'a>(
        &self, &'a scene::Scene, &ray::RayDifferential,
        &Sample, &mut RNG) -> (Spectrum, Option<Intersection>, Spectrum);

    fn li_simple(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &Sample, rng: &mut RNG) -> Spectrum {
        self.li(scene, ray, sample, rng).0
    }

    fn transmittance(
        &self, &scene::Scene, &ray::RayDifferential,
        &Sample, &mut RNG) -> Spectrum;

    // Rnderer Interface
}
