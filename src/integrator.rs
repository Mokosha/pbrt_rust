use camera;
use renderer;
use scene;

pub struct SurfaceIntegrator;
pub struct VolumeIntegrator;

pub trait Integrator {
    fn preprocess(&mut self, &scene::Scene, &camera::Camera);
}

impl Integrator for SurfaceIntegrator {
    fn preprocess(&mut self, scene: &scene::Scene, cam: &camera::Camera) { }
}

impl Integrator for VolumeIntegrator {
    fn preprocess(&mut self, scene: &scene::Scene, cam: &camera::Camera) { }
}
