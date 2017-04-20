use std::sync::Arc;

use bsdf::BSDF;
use bsdf::fresnel::Fresnel;
use bsdf::lambertian::Lambertian;
use bsdf::microfacet::Microfacet;
use bsdf::microfacet::MicrofacetDistribution;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::Texture;
use utils::Clamp;

use material::bump;

#[derive(Clone, Debug)]
pub struct PlasticMaterial {
    k_d: Arc<Texture<Spectrum>>,
    k_s: Arc<Texture<Spectrum>>,
    roughness: Arc<Texture<f32>>,
    bump_map: Option<Arc<Texture<f32>>>
}

impl PlasticMaterial {
    pub fn new(kd: Arc<Texture<Spectrum>>,
               ks: Arc<Texture<Spectrum>>,
               rough: Arc<Texture<f32>>,
               bm: Option<Arc<Texture<f32>>>) -> PlasticMaterial {
        PlasticMaterial {
            k_d: kd, k_s: ks, roughness: rough, bump_map: bm
        }
    }

    pub fn get_bsdf(&self, dg_geom: DifferentialGeometry,
                    dg_shading: DifferentialGeometry) -> Option<BSDF> {
        // Allocate bsdf possibly doing bump mapping with bump map
        let dgs = if let Some(ref tex) = self.bump_map {
            bump(tex, &dg_geom, &dg_shading)
        } else {
            dg_shading
        };

        let mut bsdf = BSDF::new(dgs.clone(), dg_geom.nn);

        let kd = self.k_d.evaluate(&dgs).clamp(0.0, ::std::f32::MAX);
        let diff = Lambertian::new(kd);
        let fresnel = Fresnel::dielectric(1.5, 1.0);

        let ks = self.k_s.evaluate(&dgs).clamp(0.0, ::std::f32::MAX);
        let rough = self.roughness.evaluate(&dgs);
        let spec = Microfacet::new(ks, fresnel, MicrofacetDistribution::blinn(1.0 / rough));

        bsdf.add_bxdf(diff);
        bsdf.add_bxdf(spec);

        Some(bsdf)
    }
}
