use geometry::vector::Vector;
use spectrum::Spectrum;
use utils::Clamp;

fn fr_diel(cosi: f32, cost: f32, etai: &Spectrum,
           etat: &Spectrum) -> Spectrum {
    let rparl =
        ((etat * cosi) - (etai * cost)) /
        ((etat * cosi) + (etai * cost));
    let rperp =
        ((etai * cosi) - (etat * cost)) /
        ((etai * cosi) + (etat * cost));

    (rparl * rparl + rperp * rperp) / 2.0
}

fn fr_cond(cosi: f32, eta: &Spectrum,
           k: &Spectrum) -> Spectrum {
    let tmp: Spectrum = (eta * eta + k * k) * cosi * cosi;
    let rparl2 = {
        let t: Spectrum = 2.0 * eta * cosi;
        (tmp - t + 1.0) / (tmp + t + 1.0)
    };

    let tmp_f: Spectrum = eta*eta + k*k;
    let rperp2 =
        (tmp_f - (2.0 * eta * cosi) + cosi * cosi) /
        (tmp_f + (2.0 * eta * cosi) + cosi * cosi);

    (rparl2 + rperp2) / 2.0
}

pub enum Fresnel {
    Conductor {
        eta: Spectrum,  // Index of refraction
        k: Spectrum     // Scattering coefficient
    },
    Dielectric {
        eta_i: f32,
        eta_t: f32
    },
    NoOp
}

impl Fresnel {
    pub fn conductor(e: &Spectrum, kk: &Spectrum) -> Fresnel {
        Fresnel::Conductor {
            eta: e.clone(),
            k: kk.clone()
        }
    }

    pub fn dielectric(ei: f32, et: f32) -> Fresnel {
        Fresnel::Dielectric {
            eta_i: ei,
            eta_t: et
        }
    }

    pub fn noop() -> Fresnel { Fresnel::NoOp }

    pub fn evaluate(&self, cosi: f32) -> Spectrum {
        match self {
            &Fresnel::Conductor { ref eta, ref k } =>
                fr_cond(cosi.abs(), eta, k),
            &Fresnel::Dielectric { eta_i, eta_t } => {
                let ci = cosi.clamp(-1.0, 1.0);

                // Compute indices of refraction for dielectric
                let mut ei = eta_i;
                let mut et = eta_t;

                if cosi <= 0.0 {
                    ::std::mem::swap(&mut ei, &mut et);
                }

                // Compute sint using Snell's law
                let sint = (ei / et) * (1.0 - ci * ci).max(0.0).sqrt();

                if sint >= 1.0 {
                    // Handle total internal reflection
                    Spectrum::from(1.0)
                } else {
                    let cost = (1.0 - sint * sint).max(0.0).sqrt();
                    fr_diel(ci.abs(), cost, &Spectrum::from(ei),
                            &Spectrum::from(et))
                }
            }
            &Fresnel::NoOp => Spectrum::from(1.0)
        }
    }
}
