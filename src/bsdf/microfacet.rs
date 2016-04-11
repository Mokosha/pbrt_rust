use bsdf;
use bsdf::BxDF;
use bsdf::fresnel::Fresnel;
use bsdf::utils::*;
use geometry::normal::Normalize;
use geometry::vector::Vector;
use geometry::vector::Dot;
use spectrum::Spectrum;
use utils::Degrees;

pub enum MicrofacetDistribution {
    Blinn(f32),
    Anisotropic(f32, f32)
}

impl MicrofacetDistribution {
    pub fn blinn(e: f32) -> MicrofacetDistribution {
        if e > 1000.0 || e.is_nan() {
            MicrofacetDistribution::Blinn(1000.0)
        } else {
            MicrofacetDistribution::Blinn(e)
        }
    }

    pub fn anisotropic(e1: f32, e2: f32) -> MicrofacetDistribution {
        let x1 = if e1 > 1000.0 || e1.is_nan() { 1000.0 } else { e1 };
        let x2 = if e2 > 1000.0 || e2.is_nan() { 1000.0 } else { e2 };
        MicrofacetDistribution::Anisotropic(x1, x2)
    }

    fn d(&self, wh: &Vector) -> f32 {
        let invtwopi = 1.0 / (2.0 * ::std::f32::consts::PI);
        match self {
            &MicrofacetDistribution::Blinn(e) => {
                let costhetah = abs_cos_theta(wh);
                (e + 2.0) * invtwopi * costhetah.powf(e)
            }
            &MicrofacetDistribution::Anisotropic(ex, ey) => {
                let costhetah = abs_cos_theta(wh);
                let d = 1.0 - (costhetah * costhetah);
                if d == 0.0 {
                    return 0.0;
                }

                let e = (ex * wh.x * wh.x + ey * wh.y * wh.y) / d;
                ((ex + 2.0) * (ey + 2.0)).sqrt() * invtwopi * costhetah.powf(e)
            }
        }
    }
}

pub struct Microfacet {
    r: Spectrum,
    distribution: MicrofacetDistribution,
    fresnel: Fresnel
}

impl Microfacet {
    pub fn new(reflectance: Spectrum, f: Fresnel,
               dist: MicrofacetDistribution) -> Microfacet {
        Microfacet {
            r: reflectance,
            distribution: dist,
            fresnel: f
        }
    }

    fn g(&self, wo: &Vector, wi: &Vector, wh: &Vector) -> f32 {
        let ndotwh = abs_cos_theta(wh);
        let ndotwo = abs_cos_theta(wo);
        let ndotwi = abs_cos_theta(wi);
        let wodotwh = wo.abs_dot(wh);
        (2.0 * ndotwh * ndotwo / wodotwh)
            .min(2.0 * ndotwh * ndotwi / wodotwh)
            .min(1.0)
    }
}

impl BxDF for Microfacet {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BSDF_REFLECTION | bsdf::BSDF_GLOSSY).contains(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        let cos_theta_o = abs_cos_theta(&wo);
        let cos_theta_i = abs_cos_theta(&wi);

        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return Spectrum::from(0.0)
        }

        let wh = (wo + wi).normalize();
        let cos_theta_h = wi.dot(&wh);
        let f = self.fresnel.evaluate(cos_theta_h);
        (self.r * self.distribution.d(&wh) * self.g(&wo, &wi, &wh) * f) /
            (4.0 * cos_theta_i * cos_theta_o)
    }

    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}
