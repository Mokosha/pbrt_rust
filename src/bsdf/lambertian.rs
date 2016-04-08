use bsdf;
use bsdf::BxDF;
use geometry::vector::Vector;
use spectrum::Spectrum;

pub struct Lambertian {
    r: Spectrum
}

impl Lambertian {
    pub fn new(k: Spectrum) -> Lambertian { Lambertian { r: k } }
}

impl BxDF for Lambertian {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BSDF_REFLECTION | bsdf::BSDF_DIFFUSE).contains(ty)
    }

    fn f(&self, _: &Vector, _: &Vector) -> Spectrum {
        let invpi = 1.0 / ::std::f32::consts::PI;
        self.r * invpi
    }

    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }

    fn rho_hd(&self, _: &Vector, _: &[f32]) -> Spectrum { self.r.clone() }

    fn rho_hh(&self, _: &[f32], _: &[f32]) -> Spectrum { self.r.clone() }
}
