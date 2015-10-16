use sampler::Sample;
use scene::Scene;
use spectrum::Spectrum;
use renderer::Renderer;
use rng::RNG;

pub struct VisibilityTester;
impl VisibilityTester {
    pub fn unoccluded(&self, scene: &Scene) -> bool { false }
    pub fn transmittance<R: Renderer, T: RNG>(
        &self, scene: &Scene, renderer: &R,
        sample: &Sample, rng: &mut T) -> Spectrum {
        Spectrum::from_value(0f32)
    }
}
