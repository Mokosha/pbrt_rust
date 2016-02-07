use std::ops::Add;
use std::ops::Sub;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;

use utils::Lerp;
use utils::Clamp;

const NUM_SPECTRUM_SAMPLES: usize = 30;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Spectrum {
    RGB([f32; 3]),
    Sampled([f32; NUM_SPECTRUM_SAMPLES])
}

impl Spectrum {

    fn sampled(cs: [f32; NUM_SPECTRUM_SAMPLES]) -> Spectrum {
        Spectrum::Sampled(cs)
    }

    fn rgb(cs: [f32; 3]) -> Spectrum {
        Spectrum::RGB(cs)
    }

    fn coeffs<'a>(&'a self) -> &'a [f32] {
        match self {
            &Spectrum::RGB(ref coeffs) => coeffs,
            &Spectrum::Sampled(ref coeffs) => coeffs
        }
    }

    fn elementwise<F>(self, _rhs: Spectrum, f: F) -> Spectrum
        where F : Fn((f32, f32)) -> f32 {
            match self {
                Spectrum::RGB(cs) => {
                    let mut _rhs_cs = match _rhs {
                        Spectrum::RGB(_cs) => _cs,
                        _ => panic!("RGB & non-RGB mismatch!")
                    };

                    for i in 0..3 {
                        _rhs_cs[i] = f((cs[i], _rhs_cs[i]));
                    }

                    Spectrum::RGB(_rhs_cs)
                },

                Spectrum::Sampled(cs) => {
                    let mut _rhs_cs = match _rhs {
                        Spectrum::Sampled(_cs) => _cs,
                        _ => panic!("Sampled & non-Sampled mismatch!")
                    };

                    for i in 0..3 {
                        _rhs_cs[i] = f((cs[i], _rhs_cs[i]));
                    }

                    Spectrum::Sampled(_rhs_cs)
                }
            }
        }

    fn transform<F>(self, f: F) -> Spectrum
        where F : Fn(f32) -> f32 {
            match self {
                Spectrum::RGB(mut cs) => {
                    for i in 0..3 {
                        cs[i] = f(cs[i]);
                    }

                    Spectrum::RGB(cs)
                },

                Spectrum::Sampled(mut cs) => {
                    for i in 0..3 {
                        cs[i] = f(cs[i]);
                    }

                    Spectrum::Sampled(cs)
                }
            }
        }

    pub fn has_nans(&self) -> bool {
        self.coeffs().iter().fold(false, |r, x| r || x.is_nan())
    }

    pub fn from_value(f: f32) -> Spectrum { Spectrum::rgb([f, f, f]) }

    pub fn is_black(&self) -> bool {
        self.coeffs().iter().fold(true, |r, x| r && *x == 0.0)
    }

    pub fn sqrt(self) -> Spectrum {
        self.transform(|x| x.sqrt())
    }

    pub fn powf(self, n: f32) -> Spectrum {
        self.transform(|x| x.powf(n))
    }

    pub fn powi(self, n: i32) -> Spectrum {
        self.transform(|x| x.powi(n))
    }

    pub fn exp(self) -> Spectrum {
        self.transform(|x| x.exp())
    }
}

impl Add for Spectrum {
    type Output = Spectrum;
    fn add(self, _rhs: Spectrum) -> Spectrum {
        self.elementwise(_rhs, |(x, y)| x + y)
    }
}

impl Sub for Spectrum {
    type Output = Spectrum;
    fn sub(self, _rhs: Spectrum) -> Spectrum {
        self.elementwise(_rhs, |(x, y)| x - y)
    }
}

impl Mul for Spectrum {
    type Output = Spectrum;
    fn mul(self, _rhs: Spectrum) -> Spectrum {
        self.elementwise(_rhs, |(x, y)| x * y)
    }
}

impl Mul<f32> for Spectrum {
    type Output = Spectrum;
    fn mul(self, _rhs: f32) -> Spectrum {
        self.transform(|x| x * _rhs)
    }
}

impl Mul<Spectrum> for f32 {
    type Output = Spectrum;
    fn mul(self, _rhs: Spectrum) -> Spectrum {
        _rhs.transform(|x| x * self)
    }
}

impl Div for Spectrum {
    type Output = Spectrum;
    fn div(self, _rhs: Spectrum) -> Spectrum {
        self.elementwise(_rhs, |(x, y)| x / y)
    }
}

impl Div<f32> for Spectrum {
    type Output = Spectrum;
    fn div(self, _rhs: f32) -> Spectrum {
        self.transform(|x| x / _rhs)
    }
}

impl Neg for Spectrum {
    type Output = Spectrum;
    fn neg(self) -> Spectrum {
        self.transform(|x| -x)
    }
}

impl Lerp<f32> for Spectrum {
    fn lerp(&self, b: &Self, t: f32) -> Self {
        self.clone().elementwise(b.clone(), |(x, y)| (&x).lerp(&y, t))
    }
}

impl Clamp<f32> for Spectrum {
    fn clamp(self, a: f32, b: f32) -> Spectrum {
        self.transform(|x| x.clamp(a, b))
    }
}
