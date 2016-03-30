use bsdf::BxDF;
use geometry::vector::Dot;
use integrator::Integrator;
use integrator::SurfaceIntegrator;
use intersection::Intersection;
use light::LightSample;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;
use sampler::sample::Sample;
use scene::Scene;
use spectrum::Spectrum;

use integrator::specular_reflect;
use integrator::specular_transmit;

#[derive(Clone, Debug)]
pub struct WhittedIntegrator {
    // WhittedIntegrator Private Data
    max_depth: usize,
}

impl WhittedIntegrator {
    pub fn new(d: usize) -> WhittedIntegrator {
        WhittedIntegrator {
            max_depth: d
        }
    }

    fn li<R : Renderer>(&self, scene: &Scene,
                        renderer: &R,
                        rayd: &RayDifferential,
                        isect: &mut Intersection,
                        sample: &Sample,
                        rng: &mut RNG) -> Spectrum {
        // Compute emitted and reflected light at ray intersection point
        // Evaluate BSDF at hit point
        let ray = &rayd.ray;
        let bsdf = isect.get_bsdf();

        // Initialize common variables for Whitted Integrator
        let p = &(bsdf.dg_shading.p);
        let n = &(bsdf.dg_shading.nn);
        let wo = -(&ray.d);

        // Compute emitted light if ray hit an area light source
        let l = scene.lights().iter().fold(isect.le(&wo), |l_acc, ref light| {

            // Add contribution of each light source
            let (li, wi, pdf, visibility) =
                light.sample_l(p, isect.ray_epsilon,
                               LightSample::new(rng), ray.time.clone());
            if li.is_black() || pdf == 0f32 { l_acc }
            else {
                let f = bsdf.f(&wo, &wi);
                if f.is_black() || !visibility.unoccluded(scene) { l_acc }
                else {
                    l_acc +
                        f * li * wi.abs_dot(n) *
                        visibility.transmittance(scene, renderer, sample, rng) / pdf
                }
            }
        });

        l + (
            if ray.depth + 1 < self.max_depth {
                // Trace rays for specular reflection and refraction
                let refl = specular_reflect(rayd, &bsdf, rng, isect,
                                            renderer, scene, sample);
                let tmit = specular_transmit(rayd, &bsdf, rng, isect,
                                             renderer, scene, sample);
                refl + tmit
            } else { Spectrum::from_value(0f32) }
        )
    }
}
