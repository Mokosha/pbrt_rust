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

    fn li_simple<T:RNG>(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &sampler::Sample, rng: &mut T,
        ) -> Spectrum {
        let mut dummy_isect = None;
        let mut dummy_spect = None;
        self.li(scene, ray, sample, rng, &mut dummy_isect, &mut dummy_spect)
    }

    fn transmittance<T:RNG>(
        &self, &scene::Scene, &ray::RayDifferential,
        &sampler::Sample, &mut T) -> Spectrum;

    // Rnderer Interface
}
