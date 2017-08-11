use spectrum::Spectrum;

#[derive(Clone, Debug, PartialEq)]
pub struct BSSRDF {
    eta: f32,
    sig_a: Spectrum,
    sigp_s: Spectrum
}

impl BSSRDF {
    pub fn new(sa: Spectrum, sps: Spectrum, et: f32) -> BSSRDF {
        BSSRDF { eta: et, sig_a: sa, sigp_s: sps }
    }

    pub fn eta(&self) -> f32 { self.eta }
    pub fn sigma_a(&self) -> Spectrum { self.sig_a }
    pub fn sigma_prime_s(&self) -> Spectrum { self.sigp_s }
}
