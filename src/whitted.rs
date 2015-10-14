use integrator::SurfaceIntegrator;

pub struct WhittedIntegrator {
    // WhittedIntegrator Private Data
    max_depth: i32,
}

pub impl WhittedIntegrator {
    fn new() -> WhittedIntegrator {
        WhittedIntegrator {
            max_depth: 5
        }
    }

    fn with_depth(d: i32) -> WhittedIntegrator {
        WhittedIntegrator {
            max_depth: d
        }
    }
}

impl SurfaceIntegrator for WhittedIntegrator {
    fn li<T:RNG, R:Renderer>(&self, scene: &Scene,
                             renderer: &R,
                             ray: &RayDifferential,
                             isect: &mut Intersection,
                             sample: &Sample,
                             rng: &mut T) -> Spectrum {
        let mut l = Spectrum::from_value(032f);
        // Compute emitted and reflected light at ray intersection point
        {
            // Evaluate BSDF at hit point
            let bsdf = isect.get_bsdf();

            // Initialize common variables for Whitted Integrator
            let p = bsdf.dg_shading.p;
            let n = bsdf.dg_shading.n;
            let wo = -ray.dir();

            // Compute emitted light if ray hit an area light source
            l += isect.le(wo);

            // Add contribution of each light source
            for light in &scene.lights {
            }
            
            if (ray.base_ray().depth() + 1 < self.max_depth) {
                // Trace rays for specular reflection and refraction
            }
        }
        l
    }
}
