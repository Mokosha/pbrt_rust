use geometry::vector::Dot;
use integrator;
use integrator::Integrator;
use integrator::SurfaceIntegrator;
use intersection::Intersection;
// use light::Light;
use light::LightSample;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;
use sampler::Sample;
use scene::Scene;
use spectrum::Spectrum;
// use visibility_tester::VisibilityTester;

pub struct WhittedIntegrator {
    // WhittedIntegrator Private Data
    max_depth: i32,
}

impl WhittedIntegrator {
    pub fn new() -> WhittedIntegrator {
        WhittedIntegrator {
            max_depth: 5
        }
    }

    pub fn with_depth(d: i32) -> WhittedIntegrator {
        WhittedIntegrator {
            max_depth: d
        }
    }
}

impl Integrator for WhittedIntegrator { }
impl SurfaceIntegrator for WhittedIntegrator {
    fn li<T : RNG, R : Renderer>(&self, scene: &Scene,
                             renderer: &R,
                             rayd: &RayDifferential,
                             isect: &mut Intersection,
                             sample: &Sample,
                             rng: &mut T) -> Spectrum {
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
                light.sample_l(p, isect.ray_epsilon, LightSample::new(rng), ray.time.clone());
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
                let refl = integrator::specular_reflect;
                let tmit = integrator::specular_transmit;
                refl(rayd, &bsdf, rng, isect, renderer, scene, sample) +
                    tmit(rayd, &bsdf, rng, isect, renderer, scene, sample)
            } else { Spectrum::from_value(0f32) }
        )
    }
}
