use bsdf;
use bsdf::BxDF;
use bsdf::fresnel::Fresnel;
use bsdf::utils::*;
use geometry::normal::Normalize;
use geometry::vector::Vector;
use geometry::vector::Dot;
use spectrum::Spectrum;
use utils::Degrees;

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
        (bsdf::BxDFType::BSDF_REFLECTION | bsdf::BxDFType::BSDF_GLOSSY).contains(ty)
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

#[derive(Debug, Clone, PartialEq)]
pub struct FresnelBlend {
    r_d: Spectrum,
    r_s: Spectrum,
    distribution: MicrofacetDistribution
}

impl FresnelBlend {
    pub fn new(rd: Spectrum, s: Spectrum,
               dist: MicrofacetDistribution) -> FresnelBlend {
        FresnelBlend {
            r_d: rd,
            r_s: s,
            distribution: dist
        }
    }

    fn schlick_fresnel(&self, cos_theta: f32) -> Spectrum {
        self.r_s + (1.0 - cos_theta).powf(5.0) * (Spectrum::from(1.0) - self.r_s)
    }
}

impl BxDF for FresnelBlend {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BxDFType::BSDF_REFLECTION | bsdf::BxDFType::BSDF_GLOSSY).contains(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        let diffuse = (28.0 / (23.0 * ::std::f32::consts::PI)) * self.r_d *
            (Spectrum::from(1.0) - self.r_s) *
            (1.0 - (1.0 - 0.5 * abs_cos_theta(wi)).powf(5.0)) *
            (1.0 - (1.0 - 0.5 * abs_cos_theta(wo)).powf(5.0));

        let wh = (wi + wo).normalize();
        let specular = self.distribution.d(&wh) /
            (4.0 * wi.abs_dot(&wh) * abs_cos_theta(wi).max(abs_cos_theta(wo))) *
            self.schlick_fresnel(wi.dot(&wh));
        diffuse + specular
    }

    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}
