use diff_geom::DifferentialGeometry;
use bsdf::BSDF;
use bsdf::BSSRDF;

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
