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
    Empty
}

impl MicrofacetDistribution {
    fn d(&self, wh: &Vector) -> f32 {
        unimplemented!()
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
