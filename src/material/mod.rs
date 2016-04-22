mod matte;

use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
use diff_geom::DifferentialGeometry;
use texture::Texture;

pub fn bump(_: &Texture<f32>, _: &DifferentialGeometry,
            _: &DifferentialGeometry) -> DifferentialGeometry {
    unimplemented!()
}

#[derive(Clone, PartialEq, Debug)]
pub struct Material;

impl Material {
    pub fn get_bsdf(&self, dg: DifferentialGeometry,
                    dgs: DifferentialGeometry) -> Option<BSDF> {
        None
    }

    pub fn get_bssrdf(&self, dg: DifferentialGeometry,
                      dgs: DifferentialGeometry) -> Option<BSSRDF> {
        None
    }
}
