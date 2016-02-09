use std::ops::Add;
use std::ops::Sub;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;

use utils::Lerp;
use utils::Clamp;

const SAMPLED_LAMBDA_START: usize = 400;
const SAMPLED_LAMBDA_END: usize = 700;
const _NUM_SPECTRUM_SAMPLES: usize = 30;

#[cfg(test)]
pub const NUM_SPECTRUM_SAMPLES: usize = _NUM_SPECTRUM_SAMPLES;
#[cfg(not(test))]
const NUM_SPECTRUM_SAMPLES: usize = _NUM_SPECTRUM_SAMPLES;

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

        if lambda_end < seg_start_lambda {
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

        sum += (wavelength_at(seg_start) + wavelength_at(seg_end))
            * 0.5 * (seg_end - seg_start);
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

                    debug_assert!(_rhs_cs.len() == cs.len());

                    for (rhs, c) in _rhs_cs.iter_mut().zip(cs.iter()) {
                        *rhs = f((*c, *rhs));
                    }

                    Spectrum::RGB(_rhs_cs)
                },

                Spectrum::Sampled(cs) => {
                    let mut _rhs_cs = match _rhs {
                        Spectrum::Sampled(_cs) => _cs,
                        _ => panic!("Sampled & non-Sampled mismatch!")
                    };

                    debug_assert!(_rhs_cs.len() == cs.len());

                    for (rhs, c) in _rhs_cs.iter_mut().zip(cs.iter()) {
                        *rhs = f((*c, *rhs));
                    }

                    Spectrum::Sampled(_rhs_cs)
                }
            }
        }

    fn transform<F>(self, f: F) -> Spectrum
        where F : Fn(f32) -> f32 {
            match self {
                Spectrum::RGB(mut cs) => {
                    for c in cs.iter_mut() {
                        *c = f(*c);
                    }

                    Spectrum::RGB(cs)
                },

                Spectrum::Sampled(mut cs) => {
                    for c in cs.iter_mut() {
                        *c = f(*c);
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

impl ::std::ops::Index<usize> for Spectrum {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match self {
            &Spectrum::Sampled(ref cs) => {
                match index {
                    0...29 => cs.get(index).unwrap(),
                    _ => panic!("Error - Sampled Spectrum index out of bounds!")
                }
            },

            &Spectrum::RGB(ref cs) => {
                match index {
                    0...2 => cs.get(index).unwrap(),
                    _ => panic!("Error - RGB Spectrum index out of bounds!")
                }
            },
        }
    }
}

impl ::std::ops::IndexMut<usize> for Spectrum {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        match self {
            &mut Spectrum::Sampled(ref mut cs) => {
                match index {
                    0...29 => cs.get_mut(index).unwrap(),
                    _ => panic!("Error - Sampled Spectrum index out of bounds!")
                }
            },

            &mut Spectrum::RGB(ref mut cs) => {
                match index {
                    0...2 => cs.get_mut(index).unwrap(),
                    _ => panic!("Error - RGB Spectrum index out of bounds!")
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32;
    use utils::Lerp;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Spectrum::from_samples(&[(400.0, 3.0); 1]),
                   Spectrum::Sampled([3f32; NUM_SPECTRUM_SAMPLES]));

        assert_eq!(Spectrum::from_value(3f32),
                   Spectrum::RGB([3f32; 3]));
    }

    #[test]
    fn it_can_be_created_from_values() {
        assert_eq!(Spectrum::from_samples(
            &[(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)]),
                   Spectrum::Sampled([3f32; NUM_SPECTRUM_SAMPLES]));

        assert_eq!(Spectrum::from_samples(
            &[(500.0, 3.0), (700.0, 3.0), (600.0, 3.0), (400.0, 3.0)]),
                   Spectrum::Sampled([3f32; NUM_SPECTRUM_SAMPLES]));

        let expected =
            [4.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0,
             5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0,
             5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 4.0];
        assert_eq!(Spectrum::from_samples(
            &[(400.0, 3.0), (410.0, 5.0), (690.0, 5.0), (700.0, 3.0)]),
                   Spectrum::Sampled(expected));
    }

    #[test]
    fn it_can_be_subtracted() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        let s2 = [(500.0, 3.0), (700.0, 3.0), (600.0, 3.0), (400.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) -
                 Spectrum::from_samples(&s2)).is_black());

        assert_eq!(Spectrum::from_value(10.0) - Spectrum::from_value(6.0),
                   Spectrum::from_value(4.0));
    }

    #[test]
    fn it_can_be_added() {
        let s1 = [(400.0, -3.0), (500.0, -3.0), (600.0, -3.0), (700.0, -3.0)];
        let s2 = [(500.0, 3.0), (700.0, 3.0), (600.0, 3.0), (400.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 Spectrum::from_samples(&s2)).is_black());

        assert_eq!(Spectrum::from_value(10.0) + Spectrum::from_value(6.0),
                   Spectrum::from_value(16.0));
    }

    #[test]
    fn it_can_be_scale() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 (-1.0) * Spectrum::from_samples(&s1)).is_black());

        assert_eq!(Spectrum::from_value(10.0) * 6.0,
                   Spectrum::from_value(10.0 * 6.0));
    }

    #[test]
    fn it_can_be_divided_by_scalars() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 (-3.0) * (Spectrum::from_samples(&s1) / 3.0)).is_black());

        assert_eq!(Spectrum::from_value(10.0) / 6.0,
                   Spectrum::from_value(10.0 / 6.0));
    }

    #[test]
    fn it_can_be_negated() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 -Spectrum::from_samples(&s1)).is_black());

        assert_eq!(-Spectrum::from_value(10.0),
                   Spectrum::from_value(-10.0));
    }

    #[test]
    fn it_can_be_indexed() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert_eq!(Spectrum::from_samples(&s1)[0], 3.0);
        assert_eq!(Spectrum::from_samples(&s1)[4], 3.0);
        assert_eq!(Spectrum::from_samples(&s1)[29], 3.0);

        let mut s2 = Spectrum::from_samples(&s1);
        assert_eq!(s2[0], 3.0);
        assert_eq!(s2[4], 3.0);
        assert_eq!(s2[29], 3.0);

        let s = Spectrum::from_value(1.0);
        for i in 0..3 {
            assert_eq!(s[i], 1.0);
        }
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert_eq!(Spectrum::from_samples(&s1)[30], 3.0);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much2() {
        assert_eq!(Spectrum::from_value(1.0)[3], 3.0);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        let mut sp = Spectrum::from_samples(&s1);
        assert_eq!(sp[30], 3.0);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either2() {
        let mut sp = Spectrum::from_value(1.0);
        assert_eq!(sp[3], 3.0);
    }

    #[test]
    fn it_can_be_interpolated() {
        let s1 = Spectrum::from_samples(&[(400.0, 3.0), (500.0, 3.0),
                                          (600.0, 3.0), (700.0, 3.0)]);
        let s2 = Spectrum::from_samples(&[(700.0, 2.0), (500.0, 2.0),
                                          (400.0, 2.0), (600.0, 2.0)]);
        assert_eq!(s1.lerp(&s2, 0.5).coeffs(), [2.5; NUM_SPECTRUM_SAMPLES]);
        assert_eq!(s1.lerp(&s2, 0.0).coeffs(), [3.0; NUM_SPECTRUM_SAMPLES]);
        assert_eq!(s1.lerp(&s2, 1.0).coeffs(), [2.0; NUM_SPECTRUM_SAMPLES]);

        let s3 = Spectrum::from_value(10.0);
        let s4 = Spectrum::from_value(6.0);
        assert_eq!(s3.lerp(&s4, 0.75).coeffs(), [7.0; 3]);
        assert_eq!(s3.lerp(&s4, 0.0).coeffs(), [10.0; 3]);
        assert_eq!(s3.lerp(&s4, 1.0).coeffs(), [6.0; 3]);
    }
}

