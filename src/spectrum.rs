use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;

trait CoefficientSpectrum {
    type CoeffType : Copy+Add+Mul;
    fn num_coefficients() -> usize;
    fn coeff(&self, idx: usize) -> Self::CoeffType;
    fn coeff_mut<'a>(&mut self, idx: usize) -> &'a mut Self::CoeffType;
}


#[derive(Debug, Copy, Clone)]
pub struct Spectrum;

impl Spectrum {
    pub fn is_valid(self) -> bool { true }

    pub fn from_value(f: f32) -> Spectrum { Spectrum }
    pub fn is_black(&self) -> bool { true }
}

fn mul(spect: Spectrum, s: f32) -> Spectrum {
    Spectrum
}

impl Mul<f32> for Spectrum {
    type Output = Spectrum;
    fn mul(self, s: f32) -> Spectrum { mul(self, s) }
}

impl Div<f32> for Spectrum {
    type Output = Spectrum;
    fn div(self, s: f32) -> Spectrum { Spectrum }
}

impl Mul<Spectrum> for Spectrum {
    type Output = Spectrum;
    fn mul(self, spectrum: Spectrum) -> Spectrum { Spectrum }
}

impl Mul<Spectrum> for f32 {
    type Output = Spectrum;
    fn mul(self, spectrum: Spectrum) -> Spectrum { mul(spectrum, self) }
}

impl Add<Spectrum> for Spectrum {
    type Output = Spectrum;
    fn add(self, spectrum: Spectrum) -> Spectrum { Spectrum }
}
