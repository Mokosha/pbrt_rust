use bsdf::fresnel::Fresnel;
use bsdf;
use bsdf::BxDF;
use bsdf::utils::*;
use geometry::vector::Vector;
use spectrum::Spectrum;

pub struct SpecularReflection {
    r: Spectrum,
    fresnel: Fresnel
}

impl SpecularReflection {
    fn new(_r: Spectrum, _f: Fresnel) -> SpecularReflection {
        SpecularReflection {
            r: _r,
            fresnel: _f
        }
    }
}

impl BxDF for SpecularReflection {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        ty.contains(bsdf::BSDF_REFLECTION | bsdf::BSDF_SPECULAR)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        // Chances that integrator sends wo as reflected direction
        // of wi are measure zero....
        Spectrum::from(0f32)
    }

    fn sample_f(&self, wo: &Vector, u1: f32,
                u2: f32) -> (Vector, f32, Spectrum) {
        // Compute perfect specular reflection direction
        let wi = Vector::new_with(-wo.x, -wo.y, wo.z);
        let v = self.fresnel.evaluate(cos_theta(wo.clone()));
        (wi.clone(), 1.0, v * self.r / abs_cos_theta(wi))
    }
}
