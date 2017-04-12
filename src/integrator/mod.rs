mod whitted;

use bsdf;
use bsdf::BxDFType;
use bsdf::BSDF;
use bsdf::BSDFSample;
use camera::Camera;
use geometry::vector::Dot;
use geometry::vector::Vector;
use intersection::Intersection;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;
use sampler::sample::Sample;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

use integrator::whitted::WhittedIntegrator;

pub fn specular_reflect<R: Renderer>(
    ray: &RayDifferential, bsdf: &BSDF,
    rng: &mut RNG, isect: &Intersection, renderer: &R,
    scene: &Scene, sample: &Sample) -> Spectrum {
    let wo = -(&ray.ray.d);
    let p = &(bsdf.dg_shading.p);
    let n = &(bsdf.dg_shading.nn);
    let (wi, pdf, f) = bsdf.sample_f(
        &wo, BSDFSample::new(rng), bsdf::BSDF_REFLECTION | bsdf::BSDF_SPECULAR);

    let win = wi.abs_dot(n);
    if pdf <= 0f32 || f.is_black() || win == 0f32 {
        return Spectrum::from(0.0);
    }

    // Compute ray differential rd for specular reflection
    let rd =
        if ray.has_differentials {
            let mut reflected_ray = ray.clone();
            reflected_ray.has_differentials = true;
            reflected_ray.rx_origin = p + &isect.dg.dpdx;
            reflected_ray.ry_origin = p + &isect.dg.dpdy;

            // Compute differential reflected directions
            let dndx =
                &bsdf.dg_shading.dndu * bsdf.dg_shading.dudx +
                &bsdf.dg_shading.dndv * bsdf.dg_shading.dvdx;
            let dndy =
                &bsdf.dg_shading.dndu * bsdf.dg_shading.dudy +
                &bsdf.dg_shading.dndv * bsdf.dg_shading.dvdy;

            let dwodx = -(&ray.rx_dir) - &wo;
            let dwody = -(&ray.ry_dir) - &wo;

            let ddndx = n.dot(&dwodx) + wo.dot(&dndx);
            let ddndy = n.dot(&dwody) + wo.dot(&dndy);

            reflected_ray.rx_dir =
                &wi - dwodx + 2.0 *
                Vector::from(n.dot(&wo) * dndx + ddndx * n);

            reflected_ray.ry_dir =
                &wi - dwody + 2.0 *
                Vector::from(n.dot(&wo) * dndy + ddndy * n);

            reflected_ray
        } else {
            ray.clone()
        };

    f * renderer.li_simple(scene, &rd, sample, rng) * win / pdf
}

pub fn specular_transmit<R: Renderer>(
    ray: &RayDifferential, bsdf: &BSDF,
    rng: &mut RNG, isect: &Intersection, renderer: &R,
    scene: &Scene, sample: &Sample) -> Spectrum {
    let wo = -(&ray.ray.d);
    let p = &(bsdf.dg_shading.p);
    let n = &(bsdf.dg_shading.nn);
    let (wi, pdf, f) = bsdf.sample_f(
        &wo, BSDFSample::new(rng),
        bsdf::BSDF_TRANSMISSION | bsdf::BSDF_SPECULAR);

    let win = wi.abs_dot(n);
    if pdf <= 0f32 || f.is_black() || win == 0f32 {
        return Spectrum::from(0.0);
    }

    // Compute ray differential rd for specular reflection
    let rd =
        if ray.has_differentials {
            let mut reflected_ray = ray.clone();
            reflected_ray.has_differentials = true;
            reflected_ray.rx_origin = p + &isect.dg.dpdx;
            reflected_ray.ry_origin = p + &isect.dg.dpdy;

            // Compute differential transmitted directions
            let dndx =
                &bsdf.dg_shading.dndu * bsdf.dg_shading.dudx +
                &bsdf.dg_shading.dndv * bsdf.dg_shading.dvdx;
            let dndy =
                &bsdf.dg_shading.dndu * bsdf.dg_shading.dudy +
                &bsdf.dg_shading.dndv * bsdf.dg_shading.dvdy;

            let dwodx = -(&ray.rx_dir) - &wo;
            let dwody = -(&ray.ry_dir) - &wo;

            let ddndx = n.dot(&dwodx) + wo.dot(&dndx);
            let ddndy = n.dot(&dwody) + wo.dot(&dndy);

            let w = ray.ray.d.clone();
            let eta = if n.dot(&wo) < 0.0 {
                1.0 / bsdf.eta
            } else {
                bsdf.eta
            };

            let mu = eta * n.dot(&w) - n.dot(&wi);
            let (dmudx, dmudy) = {
                let f = eta - (eta * eta * n.dot(&w)) / n.dot(&wi);
                (f * ddndx, f * ddndy)
            };

            reflected_ray.rx_dir =
                &wi + eta * dwodx - Vector::from(mu * dndx + dmudx * n);

            reflected_ray.ry_dir =
                &wi + eta * dwody - Vector::from(mu * dndy + dmudy * n);

            reflected_ray
        } else {
            ray.clone()
        };

    f * renderer.li_simple(scene, &rd, sample, rng) * win / pdf
}

#[derive(Clone, Debug)]
pub struct Integrator;

impl Integrator {
    fn preprocess(&mut self, scene: &Scene, camera: &Camera) {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub enum SurfaceIntegrator {
    Whitted {
        base: Integrator,
        surf: WhittedIntegrator
    }
}

impl SurfaceIntegrator {
    pub fn whitted(max_depth: usize) -> SurfaceIntegrator {
        SurfaceIntegrator::Whitted {
            base: Integrator,
            surf: WhittedIntegrator::new(max_depth)
        }
    }

    pub fn li<R:Renderer>(
        &self, _: &Scene, _: &R, _: &RayDifferential,
        _: &mut Intersection, _: &Sample, _: &mut RNG) -> Spectrum {
        unimplemented!()
    }

    pub fn preprocess(&mut self, scene: &Scene, camera: &Camera) {
        match self {
            &mut SurfaceIntegrator::Whitted { ref mut base, .. } =>
                base.preprocess(scene, camera)
        }
    }

    pub fn request_samples(&self, _: &Sampler, _: &mut Sample, _: &Scene) {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct VolumeIntegrator {
    base: Integrator
}

impl VolumeIntegrator {
    pub fn li<R:Renderer>(
        &self, _: &Scene, _: &R, _: &RayDifferential,
        _: &Sample, _: &mut RNG, _: &mut Spectrum) -> Spectrum {
        unimplemented!()
    }

    pub fn preprocess(&mut self, scene: &Scene, camera: &Camera) {
        self.base.preprocess(scene, camera);
    }

    pub fn request_samples(&self, _: &Sampler, _: &mut Sample, _: &Scene) {
        unimplemented!()
    }
}
