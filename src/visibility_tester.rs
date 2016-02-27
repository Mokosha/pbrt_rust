use sampler::sample::Sample;
use scene::Scene;
use spectrum::Spectrum;
use renderer::Renderer;
use rng::RNG;

pub struct VisibilityTester;
impl VisibilityTester {
    pub fn unoccluded(&self, scene: &Scene) -> bool { false }
    pub fn transmittance<R: Renderer>(
        &self, scene: &Scene, renderer: &R,
        sample: &Sample, rng: &mut RNG) -> Spectrum {
        Spectrum::from_value(0f32)
    }
}
