use std::ops::Add;
use std::ops::Sub;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;

use utils::Lerp;
use utils::Clamp;

const SAMPLED_LAMBDA_START: usize = 400;
const SAMPLED_LAMBDA_END: usize = 700;
const NUM_SPECTRUM_SAMPLES: usize = 30;

fn average_spectrum_samples(samples: &[(f32, f32)],
                            lambda_start: f32,
                            lambda_end: f32) -> f32 {
    // Handle cases with out of bounds range or single sample only
    if samples.len() == 0 { return 0.0 }
    if lambda_end <= samples[0].0 { return samples[0].1 }
    if lambda_start >= samples.last().unwrap().0 {
        return samples.last().unwrap().1
    }
    if samples.len() == 1 { return samples[0].1 }

    let mut sum = 0f32;

    // Add contributions of constant segments before/after samples
    if lambda_start < samples[0].0 {
        sum += samples[0].1 * (samples[0].0 - lambda_start);
    }

    if lambda_end > samples.last().unwrap().0 {
        let lst = samples.last().unwrap();
        sum += lst.1 * (lambda_end - lst.0);
    }

    // Loop over wavelength sample segments and add contriubtions
    for (&(seg_start_lambda, seg_start_v),
         &(seg_end_lambda, seg_end_v)) in samples.iter().zip(samples.iter().skip(1)) {

        if seg_end_lambda < lambda_start {
            continue;
        }

        if lambda_end > seg_start_lambda {
            break;
        }

        let seg_start = seg_start_lambda.max(lambda_start);
        let seg_end = seg_end_lambda.min(lambda_end);

        debug_assert!(seg_start >= seg_start_lambda);
        debug_assert!(seg_start >= lambda_start);
        debug_assert!(seg_end <= seg_end_lambda);
        debug_assert!(seg_end <= lambda_end);

        let wavelength_at = |w| {
            let t = (w - seg_start_lambda) / (seg_end_lambda - seg_start_lambda);
            seg_start_v.lerp(&seg_end_v, t)
        };

        sum += (wavelength_at(seg_start) + wavelength_at(seg_end)) * 0.5 * (seg_end - seg_start);
    }

    return sum / (lambda_end - lambda_start);
}

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

    pub fn from_samples(samples: &[(f32, f32)]) -> Spectrum {
        // Sort samples if unordered, use sorted for returned spectrum
        let sorted = samples.iter().fold((true, ::std::f32::MIN), |(r, x), y| {
            (if y.0 < x { false } else { r }, y.0)
        }).0;

        if !sorted {
            let mut svec = samples.to_vec();
            svec.sort_by(|&(x, _), &(y, _)| x.partial_cmp(&y).unwrap() );
            return Spectrum::from_samples(&svec);
        }

        let mut cs = [0f32; NUM_SPECTRUM_SAMPLES];
        for i in 0..NUM_SPECTRUM_SAMPLES {
            // Compute average value of given SPD over ith sample's range
            let minl = SAMPLED_LAMBDA_START as f32;
            let maxl = SAMPLED_LAMBDA_END as f32;
            let ns = NUM_SPECTRUM_SAMPLES as f32;
            let lambda0 = minl.lerp(&maxl, (i as f32) / ns);
            let lambda1 = minl.lerp(&maxl, ((i + 1) as f32) / ns);

            cs[i] = average_spectrum_samples(samples, lambda0, lambda1);
        }

        Spectrum::sampled(cs)
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
