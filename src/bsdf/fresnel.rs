use geometry::vector::Vector;
use spectrum::Spectrum;

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
