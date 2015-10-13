use camera::Camera;
use intersection::Intersection;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;
use sampler::Sample;
use scene::Scene;
use spectrum::Spectrum;

pub trait Integrator {
    fn preprocess(&mut self, scene: &Scene, camera: &Camera) { }
}

pub trait SurfaceIntegrator : Integrator {
    fn li<T:RNG, R:Renderer>(&self, &Scene, &R, &RayDifferential,
                             &mut Intersection, &Sample, &mut T) -> Spectrum;
}

pub trait VolumeIntegrator : Integrator {
    fn li<T:RNG, R:Renderer>(&self, &Scene, &R, &RayDifferential,
                             &Sample, &mut T, &mut Spectrum) -> Spectrum;
}


#[derive(Debug, Clone)]
pub struct EmptyIntegrator;

impl EmptyIntegrator {
    pub fn new() -> EmptyIntegrator { EmptyIntegrator }
}

impl Integrator for EmptyIntegrator { }
impl SurfaceIntegrator for EmptyIntegrator {
    fn li<T:RNG, R:Renderer>(&self, _: &Scene, _: &R, _: &RayDifferential,
                             _: &mut Intersection, _: &Sample, _: &mut T) -> Spectrum {
        Spectrum::from_value(0f32)
    }
}

impl VolumeIntegrator for EmptyIntegrator {
    fn li<T:RNG, R:Renderer>(&self, _: &Scene, _: &R, _: &RayDifferential,
                             _: &Sample, _: &mut T, _: &mut Spectrum) -> Spectrum {
        Spectrum::from_value(0f32)
    }
}
