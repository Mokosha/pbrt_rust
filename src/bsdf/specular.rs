use bsdf::fresnel::Fresnel;
use bsdf;
use bsdf::BxDF;
use bsdf::utils::*;
use geometry::vector::Vector;
use spectrum::Spectrum;

#[derive(Clone, Debug, PartialEq)]
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
        (bsdf::BSDF_REFLECTION | bsdf::BSDF_SPECULAR).contains(ty)
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

#[derive(Clone, Debug, PartialEq)]
pub struct SpecularTransmission {
    t: Spectrum,
    etai: f32,
    etat: f32,
    fresnel: Fresnel
}

impl SpecularTransmission {
    fn new(_t: Spectrum, _etai: f32, _etat: f32) -> SpecularTransmission {
        SpecularTransmission {
            t: _t,
            etai: _etai,
            etat: _etat,
            fresnel: Fresnel::dielectric(_etai, _etat)
        }
    }
}

impl BxDF for SpecularTransmission {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BSDF_TRANSMISSION | bsdf::BSDF_SPECULAR).contains(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        // Chances here are zero, too
        Spectrum::from(0f32)
    }

    fn sample_f(&self, wo: &Vector, u1: f32,
                u2: f32) -> (Vector, f32, Spectrum) {
        let ct = cos_theta(wo.clone());

        // Figure out which eta is incident and which is transmitted
        let entering = ct > 0f32;
        let mut ei = self.etai;
        let mut et = self.etat;
        if entering {
            ::std::mem::swap(&mut ei, &mut et);
        }

        // Computed transmitted ray direction
        let sini2 = sin_theta(wo.clone());
        let eta = ei / et;
        let sint2 = eta * eta * sini2;

        // Handle total internal reflection for transmission
        if sint2 >= 1f32 {
            return (Vector::new(), 1f32, Spectrum::from(0f32))
        }

        let cost = (1f32 - sint2).max(0.0).sqrt() * (if entering { -1.0 } else { 1.0 });
        let sint_over_sini = eta;
        let wi = Vector::new_with(sint_over_sini * -wo.x, sint_over_sini * -wo.y, cost);

        let pdf = 1f32;
        let f = self.fresnel.evaluate(ct);
        let v = (et * et) / (ei * ei) * (Spectrum::from(1f32) - self.t);
        (wi.clone(), pdf, v / abs_cos_theta(wi))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bsdf;
    use bsdf::BxDF;
    use bsdf::fresnel::Fresnel;
    use geometry::vector::Vector;
    use spectrum::Spectrum;

    #[test]
    fn spec_refl_can_be_created() {
        let brdf1 = SpecularReflection::new(Spectrum::from(1f32),
                                            Fresnel::dielectric(1.0, 1.0));

        let brdf2 = SpecularReflection::new(Spectrum::from(1f32),
                                            Fresnel::dielectric(1.0, 1.0));

        assert_eq!(brdf1, brdf2);
    }

    #[test]
    fn spec_refl_is_spec_and_refl() {
        let brdf = SpecularReflection::new(Spectrum::from(1f32),
                                           Fresnel::dielectric(1.0, 1.0));
        assert!(brdf.matches_flags(bsdf::BSDF_SPECULAR));
        assert!(brdf.matches_flags(bsdf::BSDF_REFLECTION));
        assert!(!brdf.matches_flags(bsdf::BSDF_TRANSMISSION));
        assert!(!brdf.matches_flags(bsdf::BSDF_DIFFUSE));
    }

    #[test]
    fn spec_refl_has_no_f() {
        let brdf = SpecularReflection::new(Spectrum::from(1f32),
                                           Fresnel::dielectric(1.0, 1.0));
        assert_eq!(brdf.f(&Vector::new(), &Vector::new()), Spectrum::from(0.0));
        assert_eq!(brdf.f(&Vector::new_with(1.0, -1.0, 0.0),
                          &Vector::new_with(10.0, -3.14, 15.0)), Spectrum::from(0.0));
    }

    #[test]
    fn spec_refl_can_sample_direction() {
        let brdf = SpecularReflection::new(Spectrum::from(1f32),
                                           Fresnel::dielectric(1.0, 1.0));

        let (wi, _, _) = brdf.sample_f(&Vector::new_with(-(2f32.sqrt()), 0.0, 2f32.sqrt()),
                                       0.0, 0.0);
        assert_eq!(wi, Vector::new_with(2f32.sqrt(), 0.0, 2f32.sqrt()));

        let (wi2, _, _) = brdf.sample_f(&Vector::new_with(2f32.sqrt(), 0.0, 2f32.sqrt()),
                                        0.0, 0.0);
        assert_eq!(wi2, Vector::new_with(-2f32.sqrt(), 0.0, 2f32.sqrt()));

        let (wi3, _, _) = brdf.sample_f(&Vector::new_with(-(2f32.sqrt()), 0.0, -2f32.sqrt()),
                                       0.0, 0.0);
        assert_eq!(wi3, Vector::new_with(2f32.sqrt(), 0.0, -2f32.sqrt()));

        let (wi4, _, _) = brdf.sample_f(&Vector::new_with(2f32.sqrt(), 0.0, -2f32.sqrt()),
                                        0.0, 0.0);
        assert_eq!(wi4, Vector::new_with(-2f32.sqrt(), 0.0, -2f32.sqrt()));
    }

    #[test]
    fn spec_trans_can_be_created() {
        let btdf1 = SpecularTransmission::new(Spectrum::from(1f32), 1.0, 1.0);
        let btdf2 = SpecularTransmission::new(Spectrum::from(1f32), 1.0, 1.0);
        assert_eq!(btdf1, btdf2);
    }

    #[test]
    fn spec_trans_is_spec_and_refl() {
        let brdf = SpecularTransmission::new(Spectrum::from(1f32), 1.0, 1.0);
        assert!(brdf.matches_flags(bsdf::BSDF_TRANSMISSION));
        assert!(brdf.matches_flags(bsdf::BSDF_SPECULAR));
        assert!(!brdf.matches_flags(bsdf::BSDF_REFLECTION));
        assert!(!brdf.matches_flags(bsdf::BSDF_DIFFUSE));
    }

    #[test]
    fn spec_trans_has_no_f() {
        let btdf = SpecularTransmission::new(Spectrum::from(1f32), 1.0, 1.0);
        assert_eq!(btdf.f(&Vector::new(), &Vector::new()), Spectrum::from(0.0));
        assert_eq!(btdf.f(&Vector::new_with(1.0, -1.0, 0.0),
                          &Vector::new_with(10.0, -3.14, 15.0)), Spectrum::from(0.0));
    }

    #[ignore]
    #[test]
    fn spec_trans_can_sample_direction() {
        unimplemented!() // Test sample_f
    }
}
