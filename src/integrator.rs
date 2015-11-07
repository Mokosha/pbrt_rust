use bsdf;
use bsdf::BSDF;
use bsdf::BSDFSample;
use camera::Camera;
use geometry::vector::Dot;
use intersection::Intersection;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;
use sampler::Sample;
use scene::Scene;
use spectrum::Spectrum;

fn process_specular<T: RNG, R: Renderer>(
    ray: &RayDifferential, bsdf: &BSDF,
    rng: &mut T, isect: &Intersection, renderer: &R,
    scene: &Scene, sample: &Sample, sample_type: u32) -> Spectrum {
    let wo = -(&ray.ray.d);
    let p = &(bsdf.dg_shading.p);
    let n = &(bsdf.dg_shading.nn);
    let (wi, pdf, f) = bsdf.sample_f(
        &wo, BSDFSample::new(rng), sample_type);

    let win = wi.abs_dot(n);
    if (pdf > 0f32 && !f.is_black() && win != 0f32) {
        // Cmpute ray differential rd for specular reflection <512>
        let rd = ray.clone(); // !FIXME! just to compile
        let li = renderer.li_simple(scene, &rd, sample, rng);
        f * li * win / pdf
    } else {
        Spectrum::from_value(0f32)
    }
}

pub fn specular_reflect<T: RNG, R: Renderer>(
    ray: &RayDifferential, bsdf: &BSDF,
    rng: &mut T, isect: &Intersection, renderer: &R,
    scene: &Scene, sample: &Sample) -> Spectrum {
    process_specular(ray, bsdf, rng, isect, renderer, scene, sample,
                     bsdf::BSDF_REFLECTION | bsdf::BSDF_SPECULAR)
}

pub fn specular_transmit<T: RNG, R: Renderer>(
    ray: &RayDifferential, bsdf: &BSDF,
    rng: &mut T, isect: &Intersection, renderer: &R,
    scene: &Scene, sample: &Sample) -> Spectrum {
    process_specular(ray, bsdf, rng, isect, renderer, scene, sample,
                     bsdf::BSDF_TRANSMISSION | bsdf::BSDF_SPECULAR)
}

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
