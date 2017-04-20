use std::sync::Arc;

use bsdf::BSDF;
use bsdf::measured::IrregIsotropic;
use bsdf::measured::IrregIsotropicSample;
use bsdf::measured::RegularHalfangle;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::Texture;
use utils::kdtree::KdTree;

use material::bump;

#[derive(Clone, Debug)]
pub struct MeasuredMaterial {
    theta_phi_data: KdTree<IrregIsotropicSample>,
    regular_halfangle_data: Vec<f32>,
    num_theta_h: usize,
    num_theta_d: usize,
    num_phi_d: usize,
    bump_map: Option<Arc<Texture<f32>>>
}

impl MeasuredMaterial {
    pub fn new(filename: String, b: Option<Arc<Texture<f32>>>) -> MeasuredMaterial {
        // If we want to follow the datatype for the measured brdf data
        // used in PBRT-v2, then we need to follow the code given in
        // materials/measured.cpp...
        unimplemented!()
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

        if self.regular_halfangle_data.len() > 0 {
            let data = self.regular_halfangle_data.clone();
            bsdf.add_bxdf(RegularHalfangle::new(self.num_theta_h,
                                                self.num_theta_d,
                                                self.num_phi_d, data));
        } else {
            bsdf.add_bxdf(IrregIsotropic::new(self.theta_phi_data.clone()));
        }

        Some(bsdf)
    }
}
