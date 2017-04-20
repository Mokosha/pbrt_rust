use std::sync::Arc;

use bsdf::BSDF;
use bsdf::lambertian::Lambertian;
use bsdf::orennayar::OrenNayar;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::Texture;
use texture::TextureBase;
use utils::Clamp;

use material::bump;

#[derive(Clone, Debug)]
pub struct MatteMaterial {
    sigma: Arc<Texture<f32>>,
    bump_map: Option<Arc<Texture<f32>>>,
    k_d: Arc<Texture<Spectrum>>
}

impl MatteMaterial {
    pub fn new(kd: Arc<Texture<Spectrum>>,
               sig: Arc<Texture<f32>>,
               bump_map: Option<Arc<Texture<f32>>>) -> MatteMaterial {
        MatteMaterial {
            sigma: sig,
            bump_map: bump_map,
            k_d: kd
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

        // Evaluate textures for Matte material and allocate BRDF
        let r = self.k_d.evaluate(&dgs).clamp(0.0, ::std::f32::MAX);
        let sig = self.sigma.evaluate(&dgs).clamp(0.0, 90.0);
        if sig == 0.0 {
            bsdf.add_bxdf(Lambertian::new(r));
        } else {
            bsdf.add_bxdf(OrenNayar::new(r, sig));
        }

        Some(bsdf)
    }
}
