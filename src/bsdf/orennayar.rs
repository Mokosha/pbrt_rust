use bsdf;
use bsdf::BxDF;
use bsdf::utils::*;
use geometry::vector::Vector;
use spectrum::Spectrum;
use utils::Degrees;

pub struct OrenNayar {
    r: Spectrum,
    a: f32,
    b: f32
}

impl OrenNayar {
    pub fn new(k: Spectrum, sig: f32) -> OrenNayar {
        let sigma = sig.as_radians();
        let sigma2 = sigma * sigma;

        let a = 1.0 - (sigma2 / (2.0 * (sigma + 0.33)));
        let b = 0.45 * sigma2 / (sigma2 + 0.09);

        OrenNayar {
            r: k,
            a: a,
            b: b
        }
    }
}

impl BxDF for OrenNayar {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BSDF_REFLECTION | bsdf::BSDF_DIFFUSE).contains(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        let sinthetai = sin_theta(&wi);
        let sinthetao = sin_theta(&wo);

        // Compute cosine term of oren-nayar model
        let maxcos = if sinthetai < 1e-4 || sinthetao < 1e-4 { 0.0 } else {
            let sinphii = sin_phi(&wi);
            let cosphii = cos_phi(&wi);

            let sinphio = sin_phi(&wo);
            let cosphio = cos_phi(&wo);

            (cosphii * cosphio + sinphii * sinphio).max(0.0)
        };

        // Compute sine and tangent terms of oren-nayar model
        let (sinalpha, tanbeta) = if abs_cos_theta(&wi) > abs_cos_theta(&wo) {
            (sinthetao, sinthetai / abs_cos_theta(&wi))
        } else {
            (sinthetai, sinthetao / abs_cos_theta(&wo))
        };

        let invpi = 1.0 / ::std::f32::consts::PI;
        self.r * invpi * (self.a + self.b * maxcos * sinalpha * tanbeta)
    }

    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}
