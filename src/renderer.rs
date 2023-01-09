use ray;
use rng::RNG;
use sampler::sample::Sample;
use spectrum::Spectrum;
use scene;
use intersection::Intersection;

pub trait Renderer {
    fn render(&mut self, scene: &scene::Scene);

    fn li<'a>(
        &self, scene: &'a scene::Scene, ray: &ray::RayDifferential,
        sample: &Sample, rng: &mut RNG) -> (Spectrum, Option<Intersection>, Spectrum);

    fn li_simple(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &Sample, rng: &mut RNG) -> Spectrum {
        self.li(scene, ray, sample, rng).0
    }

    fn transmittance(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &Sample, rng: &mut RNG) -> Spectrum;

    // Rnderer Interface
}
