use diff_geom::DifferentialGeometry;
use bsdf::BSDF;
use bsdf::BSSRDF;

#[derive(Clone, PartialEq, Debug)]
pub struct Material;

impl Material {
    pub fn get_bsdf<'a>(&self, dg: DifferentialGeometry<'a>,
                        dgs: DifferentialGeometry<'a>) -> Option<BSDF<'a>> {
        None
    }

    pub fn get_bssrdf<'a>(&self, dg: DifferentialGeometry<'a>,
                          dgs: DifferentialGeometry<'a>) -> Option<BSSRDF<'a>> {
        None
    }
}
