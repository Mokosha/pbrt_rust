mod matte;
mod measured;
mod mix;
mod plastic;

use std::sync::Arc;

use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::Texture;

use material::matte::MatteMaterial;

pub fn bump(_: &Texture<f32>, _: &DifferentialGeometry,
            _: &DifferentialGeometry) -> DifferentialGeometry {
    unimplemented!()
}

#[derive(Clone, PartialEq, Debug)]
pub enum Material {
    Matte(MatteMaterial),
    Broken
}

impl Material {
    pub fn matte(kd: Arc<Texture<Spectrum>>,
                 sig: Arc<Texture<f32>>,
                 bump_map: Option<Arc<Texture<f32>>>) -> Material {
        Material::Matte(MatteMaterial::new(kd, sig, bump_map))
    }

    // !FIXME!
    pub fn broken() -> Material { Material::Broken }

    pub fn get_bsdf(&self, dg: DifferentialGeometry,
                    dgs: DifferentialGeometry) -> Option<BSDF> {
        match self {
            &Material::Matte(ref mat) => mat.get_bsdf(dg, dgs),
            _ => unimplemented!()
        }
    }

    pub fn get_bssrdf(&self, dg: DifferentialGeometry,
                      dgs: DifferentialGeometry) -> Option<BSSRDF> {
        None
    }
}
