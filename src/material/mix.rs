use std::sync::Arc;

use bsdf::BSDF;
use bsdf::lambertian::Lambertian;
use bsdf::orennayar::OrenNayar;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::Texture;

use material::Material;

#[derive(Clone, Debug)]
pub struct MixMaterial {
    m1: Arc<Material>,
    m2: Arc<Material>,
    scale: Arc<Texture<Spectrum>>
}

impl MixMaterial {
    pub fn new(m1: Arc<Material>, m2: Arc<Material>,
               sc: Arc<Texture<Spectrum>>) -> MixMaterial {
        MixMaterial { m1: m1, m2: m2, scale: sc }
    }

    pub fn get_bsdf(&self, dg_geom: DifferentialGeometry,
                    dg_shading: DifferentialGeometry) -> Option<BSDF> {
        let b1 = if let Some(b) = self.m1.get_bsdf(dg_geom.clone(), dg_shading.clone()) {
            b
        } else { return None };

        let b2 = if let Some(b) = self.m2.get_bsdf(dg_geom.clone(), dg_shading.clone()) {
            b
        } else { return None };

        let s = self.scale.evaluate(&dg_shading).clamp(0.0, ::std::f32::MAX);

        Some(b1.mix_with(b2, s))
    }
}
