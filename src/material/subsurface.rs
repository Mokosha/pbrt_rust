use std::sync::Arc;

use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
use bsdf::fresnel::Fresnel;
use bsdf::specular::SpecularReflection;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::Texture;

use material::bump;

#[derive(Clone, Debug)]
pub struct SubsurfaceMaterial {
    scale: f32,
    k_r: Arc<Texture<Spectrum>>,
    sigma_a: Arc<Texture<Spectrum>>,
    sigma_prime_s: Arc<Texture<Spectrum>>,
    eta: Arc<Texture<f32>>,
    bump_map: Option<Arc<Texture<f32>>>
}

impl SubsurfaceMaterial {
    pub fn new(scale: f32, k_r: Arc<Texture<Spectrum>>,
               sigma_a: Arc<Texture<Spectrum>>,
               sigma_prime_s: Arc<Texture<Spectrum>>,
               eta: Arc<Texture<f32>>,
               bump_map: Option<Arc<Texture<f32>>>) -> SubsurfaceMaterial {
        SubsurfaceMaterial { scale, k_r, sigma_a, sigma_prime_s, eta, bump_map }
    }

    pub fn get_bsdf(&self, dg_geom: DifferentialGeometry,
                    dg_shading: DifferentialGeometry) -> Option<BSDF> {
        let r = self.k_r.evaluate(&dg_geom).clamp(0.0, 1.0);
        if r.is_black() {
            return None;
        }

        // Allocate bsdf possibly doing bump mapping with bump map
        let dgs = if let Some(ref tex) = self.bump_map {
            bump(tex, &dg_geom, &dg_shading)
        } else {
            dg_shading
        };

        let fresnel = Fresnel::dielectric(1.0, self.eta.evaluate(&dg_geom));
        let mut bsdf = BSDF::new(dgs.clone(), dg_geom.nn);
        bsdf.add_bxdf(SpecularReflection::new(r, fresnel));
        Some(bsdf)
    }

    pub fn get_bssrdf(&self, _: DifferentialGeometry,
                      dgs: DifferentialGeometry) -> BSSRDF {
        BSSRDF::new(self.scale * self.sigma_a.evaluate(&dgs),
                    self.scale * self.sigma_prime_s.evaluate(&dgs),
                    self.eta.evaluate(&dgs))
    }

}
