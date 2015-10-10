use camera::Camera;
use intersection::Intersection;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;
use sampler::Sample;
use scene::Scene;
use spectrum::Spectrum;

pub struct SurfaceIntegrator;
impl SurfaceIntegrator {
    pub fn li<T:RNG, R:Renderer>(&self, scene: &Scene,
                                 renderer: &R,
                                 ray: &RayDifferential,
                                 isect: &mut Intersection,
                                 sample: &Sample,
                                 rng: &mut T) -> Spectrum {
        Spectrum
    }
}

pub struct VolumeIntegrator;
impl VolumeIntegrator {
    pub fn li<T:RNG, R:Renderer>(&self, scene: &Scene,
                                 renderer: &R,
                                 ray: &RayDifferential,
                                 sample: &Sample,
                                 rng: &mut T,
                                 transmittance: &mut Spectrum) -> Spectrum {
        Spectrum
    }
}

pub trait Integrator {
    fn preprocess(&mut self, &Scene, &Camera);
}

impl Integrator for SurfaceIntegrator {
    fn preprocess(&mut self, scene: &Scene, cam: &Camera) { }
}

impl Integrator for VolumeIntegrator {
    fn preprocess(&mut self, scene: &Scene, cam: &Camera) { }
}
