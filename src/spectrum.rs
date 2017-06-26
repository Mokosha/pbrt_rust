use std::ops::Add;
use std::ops::Sub;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;

use utils::Lerp;
use utils::Clamp;

const SAMPLED_LAMBDA_START: usize = 400;
const SAMPLED_LAMBDA_END: usize = 700;

const SAMPLED_INTEGRAL_FACTOR: f32 =
    ((SAMPLED_LAMBDA_END - SAMPLED_LAMBDA_START) as f32)
    / (NUM_SPECTRUM_SAMPLES as f32);

// Note: if you change this number you will need
// to re-run compute_xyz from the bin folder.
const _NUM_SPECTRUM_SAMPLES: usize = 30;

#[cfg(test)]
pub const NUM_SPECTRUM_SAMPLES: usize = _NUM_SPECTRUM_SAMPLES;
#[cfg(not(test))]
const NUM_SPECTRUM_SAMPLES: usize = _NUM_SPECTRUM_SAMPLES;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpectrumType {
    Reflectance,
    Illumination
}

pub fn xyz_to_rgb(xyz: [f32; 3]) -> [f32; 3] {
    [3.240479*xyz[0] - 1.37150*xyz[1] - 0.498535*xyz[2],
    -0.969256*xyz[0] + 1.875991*xyz[1] + 0.041556*xyz[2],
     0.055648*xyz[0] - 0.204043*xyz[1] + 1.057311*xyz[2]]
}

fn rgb_to_xyz(rgb: [f32; 3]) -> [f32; 3] {
    [0.412453*rgb[0] + 0.357580*rgb[1] + 0.180423*rgb[2],
     0.212671*rgb[0] + 0.715160*rgb[1] + 0.072169*rgb[2],
     0.019334*rgb[0] + 0.119193*rgb[1] + 0.950227*rgb[2]]
}

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

fn rgb_to_samples_spectrum(rgb: [f32; 3], ty: SpectrumType) -> Spectrum {
    let mut r = Spectrum::Sampled([0.0; NUM_SPECTRUM_SAMPLES]);
    match ty {
        SpectrumType::Reflectance => {
            // Convert reflectance spectrum to RGB
            if rgb[0] <= rgb[1] && rgb[0] <= rgb[2] {
                // Compute reflectance SampledSpectrum with rgb[0] as minimum
                r = r + rgb[0] * RGBREFL2SPECTWHITE;
                if rgb[1] <= rgb[2] {
                    r = r + (rgb[1] - rgb[0]) * RGBREFL2SPECTCYAN;
                    r = r + (rgb[2] - rgb[1]) * RGBREFL2SPECTBLUE;
                } else {
                    r = r + (rgb[2] - rgb[0]) * RGBREFL2SPECTCYAN;
                    r = r + (rgb[1] - rgb[2]) * RGBREFL2SPECTGREEN;
                }
            } else if rgb[1] <= rgb[0] && rgb[1] <= rgb[2] {
                // Compute reflectance _SampledSpectrum_ with rgb[1] as minimum
                r = r + rgb[1] * RGBREFL2SPECTWHITE;
                if rgb[0] <= rgb[2] {
                    r = r + (rgb[0] - rgb[1]) * RGBREFL2SPECTMAGENTA;
                    r = r + (rgb[2] - rgb[0]) * RGBREFL2SPECTBLUE;
                } else {
                    r = r + (rgb[2] - rgb[1]) * RGBREFL2SPECTMAGENTA;
                    r = r + (rgb[0] - rgb[2]) * RGBREFL2SPECTRED;
                }
            } else {
                // Compute reflectance SampledSpectrum with rgb[2] as minimum
                r = r + rgb[2] * RGBREFL2SPECTWHITE;
                if rgb[0] <= rgb[1] {
                    r = r + (rgb[0] - rgb[2]) * RGBREFL2SPECTYELLOW;
                    r = r + (rgb[1] - rgb[0]) * RGBREFL2SPECTGREEN;
                } else {
                    r = r + (rgb[1] - rgb[2]) * RGBREFL2SPECTYELLOW;
                    r = r + (rgb[0] - rgb[1]) * RGBREFL2SPECTRED;
                }
            }
            r = r * 0.94;
        },
        SpectrumType::Illumination => {
            // Convert illuminant spectrum to RGB
            if rgb[0] <= rgb[1] && rgb[0] <= rgb[2] {
                // Compute illuminant _SampledSpectrum_ with _rgb[0]_ as minimum
                r = r + rgb[0] * RGBILLUM2SPECTWHITE;
                if rgb[1] <= rgb[2] {
                    r = r + (rgb[1] - rgb[0]) * RGBILLUM2SPECTCYAN;
                    r = r + (rgb[2] - rgb[1]) * RGBILLUM2SPECTBLUE;
                } else {
                    r = r + (rgb[2] - rgb[0]) * RGBILLUM2SPECTCYAN;
                    r = r + (rgb[1] - rgb[2]) * RGBILLUM2SPECTGREEN;
                }
            } else if rgb[1] <= rgb[0] && rgb[1] <= rgb[2] {
                // Compute illuminant _SampledSpectrum_ with _rgb[1]_ as minimum
                r = r + rgb[1] * RGBILLUM2SPECTWHITE;
                if rgb[0] <= rgb[2] {
                    r = r + (rgb[0] - rgb[1]) * RGBILLUM2SPECTMAGENTA;
                    r = r + (rgb[2] - rgb[0]) * RGBILLUM2SPECTBLUE;
                } else {
                    r = r + (rgb[2] - rgb[1]) * RGBILLUM2SPECTMAGENTA;
                    r = r + (rgb[0] - rgb[2]) * RGBILLUM2SPECTRED;
                }
            } else {
                // Compute illuminant _SampledSpectrum_ with _rgb[2]_ as minimum
                r = r + rgb[2] * RGBILLUM2SPECTWHITE;
                if rgb[0] <= rgb[1] {
                    r = r + (rgb[0] - rgb[2]) * RGBILLUM2SPECTYELLOW;
                    r = r + (rgb[1] - rgb[0]) * RGBILLUM2SPECTGREEN;
                } else {
                    r = r + (rgb[1] - rgb[2]) * RGBILLUM2SPECTYELLOW;
                    r = r + (rgb[0] - rgb[1]) * RGBILLUM2SPECTRED;
                }
            }
            r = r * 0.86445;
        }
    }
    r.clamp(0.0, ::std::f32::MAX)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Spectrum {
    RGB([f32; 3]),
    Sampled([f32; NUM_SPECTRUM_SAMPLES])
}

////////////////////////////////////////////////////////////////////////////////
//
// Constants for converting between spectra
//

const SAMPLED_CIE_X: Spectrum = Spectrum::Sampled([
    0.02528201, 0.08148323, 0.21249096, 0.32426757, 0.3460886, 0.3168419,
    0.24801293, 0.14324336, 0.060038872, 0.015978936, 0.003980267, 0.031583928,
    0.11127826, 0.22651139, 0.36044234, 0.51272094, 0.67838717, 0.8414046,
    0.97610235, 1.0522884, 1.0409491, 0.93508226, 0.750358, 0.5429265, 0.362463,
    0.22055033, 0.12287091, 0.064803265, 0.0335514, 0.016247809]);

const SAMPLED_CIE_Y: Spectrum = Spectrum::Sampled([
    0.00070009334, 0.0023196535, 0.007460834, 0.017003465, 0.030045602,
    0.04834793, 0.074440464, 0.113446735, 0.17076382, 0.260841, 0.4090647,
    0.6076807, 0.790906, 0.9125535, 0.978302, 0.99830306, 0.9768859, 0.91392404,
    0.815348, 0.69457865, 0.5668466, 0.44149598, 0.32167464, 0.21798798,
    0.13915467, 0.08240787, 0.04522627, 0.023648031, 0.012163259, 0.00587135]);

const SAMPLED_CIE_Z: Spectrum = Spectrum::Sampled([
    0.12022422, 0.39005747, 1.0293049, 1.6035757, 1.7748283, 1.7357556,
    1.5092261, 1.0445968, 0.62422955, 0.3584742, 0.21300009, 0.113990866,
    0.05820007, 0.030324265, 0.013784799, 0.005946, 0.002835, 0.0018270003,
    0.0013880001, 0.0009753, 0.0005844333, 0.0002477, 0.00010853333,
    0.000031566666, 0.000009833333, 0.0, 0.0, 0.0, 0.0, 0.0]);

const CIE_Y_INT: f32 = 106.856895;

const RGBREFL2SPECTWHITE: Spectrum = Spectrum::Sampled([
    1.061683, 1.0622234, 1.06226, 1.0624601, 1.0624173, 1.0624563, 1.0624939,
    1.0624573, 1.0622259, 1.0617042, 1.0612051, 1.0611502, 1.0613848, 1.0615357,
    1.0620475, 1.0624679, 1.0624595, 1.0624985, 1.0624596, 1.0624564, 1.0625156,
    1.0625448, 1.0624843, 1.0623882, 1.0624, 1.0623133, 1.0611931, 1.0597727,
    1.0598643, 1.0601715]);

const RGBREFL2SPECTCYAN: Spectrum = Spectrum::Sampled([
    1.0196111, 1.0279436, 1.0156075, 1.0388072, 1.0446901, 1.0499041, 1.028372,
    1.0352796, 1.049238, 1.0533085, 1.053577, 1.0535184, 1.0534983, 1.0528343,
    1.0532894, 1.0548089, 1.0547378, 1.0351121, 0.7535366, 0.35679677,
    0.08360619, -0.004348372, -0.0027536866, -0.005880162, -0.001831983,
    0.002218518, 0.009080236, -0.00012466562, 0.011695187, 0.008553784]);

const RGBREFL2SPECTMAGENTA: Spectrum = Spectrum::Sampled([
    0.9870108, 1.0011885, 1.0177246, 1.0176493, 1.019203, 1.002496, 1.0064269,
    1.0145798, 0.80277836, 0.33079067, 0.0053204503, 0.0052994513, 0.002234713,
    -0.0015564138, -0.006509631, 0.0010860341, 0.011083454, 0.17798272,
    0.504212, 0.8378099, 0.9734128, 0.99143064, 1.0105921, 0.98504734,
    0.9297136, 0.8749728, 0.93711793, 0.9510805, 0.97975063, 0.9029217]);

const RGBREFL2SPECTYELLOW: Spectrum = Spectrum::Sampled([
    -0.005602702, -0.0062989057, -0.005354368, -0.00028462158, 0.02022497,
    0.08500982, 0.1839315, 0.3113346, 0.46374542, 0.63884187, 0.8147172,
    0.95972824, 1.0436108, 1.0510406, 1.0511963, 1.0511369, 1.0516016,
    1.0516258, 1.0513254, 1.0511742, 1.0514176, 1.0515511, 1.0514716, 1.0514672,
    1.0512439, 1.0513868, 1.0509636, 1.0506767, 1.0485315, 1.0487616]);

const RGBREFL2SPECTRED: Spectrum = Spectrum::Sampled([
    0.12091233, 0.10613938, 0.073351875, 0.03197698, -0.0018756868, 0.011413388,
    0.009007271, 0.010630801, 0.0024185088, -0.0039609033, -0.0053341016,
    -0.0080201775, -0.0051400284, -0.009814471, -0.0074907066, -0.002192416,
    0.0043868683, 0.014355542, 0.4147492, 0.8364955, 0.9911987, 0.9981912,
    0.9998499, 0.9945054, 1.0008614, 1.0039005, 0.98926413, 1.001913,
    0.98273456, 0.9813326]);

const RGBREFL2SPECTGREEN: Spectrum = Spectrum::Sampled([
    -0.011501977, -0.010279307, -0.011527026, -0.008418554, -0.008092569,
    -0.005521814, 0.052699924, 0.28422362, 0.600244, 0.8549946, 0.9772393,
    0.99857616, 0.9997524, 0.9995267, 0.99981654, 0.999441, 0.99693125,
    0.9599918, 0.7327138, 0.40666056, 0.13002683, 0.00419009, -0.0034842708,
    -0.0051372265, -0.0072290553, -0.008795304, -0.008621532, -0.00837295,
    -0.0076813595, -0.0022456362]);

const RGBREFL2SPECTBLUE: Spectrum = Spectrum::Sampled([
    0.99524844, 0.99449813, 0.99349815, 0.9992875, 0.99978864, 0.9991361,
    0.9845686, 0.8559035, 0.6586872, 0.4494533, 0.25420895, 0.10138621,
    0.017749699, 0.0010013992, -0.00038438066, -0.00021851463, 0.0014540203,
    0.003152723, 0.00085021195, -0.0002316068, 0.0038842566, 0.015358163,
    0.029936602, 0.04097458, 0.0489749, 0.049621098, 0.048723493, 0.040910564,
    0.03228935, 0.023743566]);

const RGBILLUM2SPECTWHITE: Spectrum = Spectrum::Sampled([
    1.1563131, 1.155765, 1.1562681, 1.1567129, 1.1567943, 1.1567554, 1.1564769,
    1.1565833, 1.1565955, 1.1564748, 1.1565808, 1.1537554, 1.1442444, 1.1338377,
    1.129764, 1.1217924, 1.0651345, 1.0455148, 1.0099921, 0.9709773, 0.9398842,
    0.9205586, 0.90967596, 0.8986726, 0.8942076, 0.8882486, 0.8827751,
    0.8801195, 0.8773263, 0.8789325]);

const RGBILLUM2SPECTCYAN: Spectrum = Spectrum::Sampled([
    1.1348593, 1.1356754, 1.135726, 1.1360767, 1.1362233, 1.1363807, 1.1357969,
    1.1360936, 1.1361942, 1.1360247, 1.1357787, 1.1357168, 1.1360912, 1.1355563,
    1.1353312, 1.1327912, 1.1038595, 0.9485222, 0.70225513, 0.4211672,
    0.19273852, 0.05010472, -0.011005207, -0.011860855, -0.011373152,
    -0.010861635, -0.006208873, -0.007594276, -0.008982474, -0.0066838684]);

const RGBILLUM2SPECTMAGENTA: Spectrum = Spectrum::Sampled([
    1.0762848, 1.0770364, 1.078422, 1.0747138, 1.0730016, 1.0735646, 1.07992,
    1.0825027, 1.010524, 0.7599703, 0.36611927, 0.06276848, 0.002039905,
    -0.0019464403, -0.0010806684, -0.00017956021, 0.0006251391, 0.018187862,
    0.18367678, 0.42199007, 0.72497827, 0.9775408, 1.0746527, 1.0814517,
    1.0558283, 1.0246029, 1.0310472, 1.0629005, 1.0084999, 1.0446938]);

const RGBILLUM2SPECTYELLOW: Spectrum = Spectrum::Sampled([
    0.00007481188, 0.0002000686, -0.00018019689, -0.0000959295, -0.00020118454,
    0.0022480635, 0.04621365, 0.33869514, 0.7971396, 1.0313917, 1.034706,
    1.0366657, 1.0365119, 1.0365998, 1.0367897, 1.0365893, 1.036445, 1.0366325,
    1.0365826, 1.0362986, 1.035497, 1.0218115, 0.9484471, 0.81737024, 0.7260127,
    0.65668875, 0.6107301, 0.59709007, 0.593377, 0.5737007]);

const RGBILLUM2SPECTRED: Spectrum = Spectrum::Sampled([
    0.059326984, 0.05409626, 0.045460433, 0.03723325, 0.024867225, 0.007980905,
    0.00069309335, 0.00043280097, 0.00062557904, -0.000018261257, -0.0002964417,
    -0.00010719348, -0.00012808296, -0.00015448067, -0.0002363224, 0.0021223067,
    0.029647697, 0.1329641, 0.29240698, 0.485022, 0.67155117, 0.81827945,
    0.9155518, 0.96905154, 0.989746, 0.99617195, 0.98861057, 0.9923029,
    0.9798074, 0.98625547]);

const RGBILLUM2SPECTGREEN: Spectrum = Spectrum::Sampled([
    0.0070419656, 0.0054942844, 0.0006513074, -0.00260272, -0.015309768,
    0.0072578825, 0.013806304, 0.21879014, 0.7212478, 1.0244777, 1.0326016,
    1.0333571, 1.0305487, 1.0198596, 1.032527, 1.036588, 1.0355723, 1.0246474,
    0.9748232, 0.38351172, -0.0019400703, 0.0034817103, 0.004575457,
    0.0065759756, 0.017162256, 0.005898682, 0.0017665531, -0.00006743472,
    -0.004299211, 0.0058450387]);

const RGBILLUM2SPECTBLUE: Spectrum = Spectrum::Sampled([
    1.0544333, 1.0542549, 1.0576292, 1.0579169, 1.0582136, 1.0579582, 1.0566978,
    1.0567003, 1.0484561, 0.69484335, 0.19655058, 0.0021658342, -0.0013501244,
    -0.0014012649, -0.001395199, -0.0015407053, 0.0004929619, -0.0008796131,
    -0.0014067921, -0.0015806735, -0.0014917275, 0.0036984228, 0.017480845,
    0.04650724, 0.09650061, 0.13728538, 0.15263234, 0.15110187, 0.16237824,
    0.16865951]);

////////////////////////////////////////////////////////////////////////////////

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

    pub fn from_rgb(rgb: [f32; 3]) -> Spectrum { Spectrum::rgb(rgb) }

    pub fn from_xyz(xyz: [f32; 3]) -> Spectrum { Spectrum::rgb(xyz_to_rgb(xyz)) }

    pub fn has_nans(&self) -> bool {
        self.coeffs().iter().fold(false, |r, x| r || x.is_nan())
    }

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

    pub fn to_xyz(&self) -> [f32; 3] {
        match self {
            &Spectrum::Sampled(ref cs) => {
                let with_spect = |spect: Spectrum| {
                    (0..NUM_SPECTRUM_SAMPLES).map(|i| {
                        spect.coeffs()[i] * cs[i]
                    }).fold(0.0, |x, y| x + y) * SAMPLED_INTEGRAL_FACTOR / CIE_Y_INT
                };

                [with_spect(SAMPLED_CIE_X),
                 with_spect(SAMPLED_CIE_Y),
                 with_spect(SAMPLED_CIE_Z)]
            },
            &Spectrum::RGB(ref rgb) => rgb_to_xyz(rgb.clone())
        }
    }

    pub fn y(&self) -> f32 {
        match self {
            &Spectrum::Sampled(ref cs) => {
                cs.iter().map(|x| *x)
                    .zip(SAMPLED_CIE_Y.coeffs().iter().map(|x| *x))
                    .map(|(x, y)| x * y)
                    .fold(0.0, |x, y| x + y) * SAMPLED_INTEGRAL_FACTOR / CIE_Y_INT
            },
            &Spectrum::RGB(ref rgb) => rgb_to_xyz(rgb.clone())[1]
        }
    }

    pub fn to_rgb(&self) -> [f32; 3] {
        match self {
            &Spectrum::Sampled(_) => xyz_to_rgb(self.to_xyz()),
            &Spectrum::RGB(rgb) => rgb.clone()
        }
    }

    pub fn into_rgb_spectrum(self) -> Spectrum {
        Spectrum::RGB(self.to_rgb())
    }

    pub fn into_sampled_spectrum(self, ty: SpectrumType) -> Spectrum {
        match self {
            Spectrum::Sampled(s) => Spectrum::Sampled(s),
            Spectrum::RGB(rgb) => rgb_to_samples_spectrum(rgb, ty)
        }
    }
}

impl ::std::convert::From<f32> for Spectrum {
    fn from(f: f32) -> Spectrum {
        Spectrum::rgb([f, f, f])
    }
}


impl Add for Spectrum {
    type Output = Spectrum;
    fn add(self, _rhs: Spectrum) -> Spectrum {
        self.elementwise(_rhs, |(x, y)| x + y)
    }
}

impl<'a> Add<&'a Spectrum> for Spectrum {
    type Output = Spectrum;
    fn add(self, _rhs: &'a Spectrum) -> Spectrum {
        self.elementwise(_rhs.clone(), |(x, y)| x + y)
    }
}

impl Add<f32> for Spectrum {
    type Output = Spectrum;
    fn add(self, _rhs: f32) -> Spectrum {
        self + Spectrum::from(_rhs)
    }
}

impl Add<Spectrum> for f32 {
    type Output = Spectrum;
    fn add(self, _rhs: Spectrum) -> Spectrum {
        Spectrum::from(self) + _rhs
    }
}

impl Sub for Spectrum {
    type Output = Spectrum;
    fn sub(self, _rhs: Spectrum) -> Spectrum {
        self.elementwise(_rhs, |(x, y)| x - y)
    }
}

impl<'a, 'b> Mul<&'b Spectrum> for &'a Spectrum {
    type Output = Spectrum;
    fn mul(self, _rhs: &'b Spectrum) -> Spectrum {
        self.clone().elementwise(_rhs.clone(),
                                 |(x, y)| x * y)
    }
}

impl<'a> Mul<Spectrum> for &'a Spectrum {
    type Output = Spectrum;
    fn mul(self, _rhs: Spectrum) -> Spectrum {
        self.clone().elementwise(_rhs, |(x, y)| x * y)
    }
}

impl<'a> Mul<&'a Spectrum> for Spectrum {
    type Output = Spectrum;
    fn mul(self, _rhs: &'a Spectrum) -> Spectrum {
        self.elementwise(_rhs.clone(), |(x, y)| x * y)
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

impl<'a> Mul<f32> for &'a Spectrum {
    type Output = Spectrum;
    fn mul(self, _rhs: f32) -> Spectrum {
        self.transform(|x| x * _rhs.clone())
    }
}

impl Mul<Spectrum> for f32 {
    type Output = Spectrum;
    fn mul(self, _rhs: Spectrum) -> Spectrum {
        _rhs.transform(|x| x * self)
    }
}

impl<'a> Mul<&'a Spectrum> for f32 {
    type Output = Spectrum;
    fn mul(self, _rhs: &'a Spectrum) -> Spectrum {
        _rhs.clone().transform(|x| x * self)
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

impl<'a> Div<f32> for &'a Spectrum {
    type Output = Spectrum;
    fn div(self, _rhs: f32) -> Spectrum {
        self.clone().transform(|x| x / _rhs)
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

impl ::std::iter::Sum<Spectrum> for Spectrum {
    fn sum<I>(iter: I) -> Spectrum where I: Iterator<Item=Spectrum> {
        iter.fold(Spectrum::rgb([0.0, 0.0, 0.0]), |acc, s| { s + acc })
    }
}

impl ::std::default::Default for Spectrum {
    fn default() -> Spectrum { Spectrum::rgb([0.0, 0.0, 0.0]) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::Lerp;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Spectrum::from_samples(&[(400.0, 3.0); 1]),
                   Spectrum::Sampled([3f32; NUM_SPECTRUM_SAMPLES]));

        assert_eq!(Spectrum::from(3f32),
                   Spectrum::RGB([3f32; 3]));
    }

    #[test]
    fn it_can_be_created_froms() {
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

        assert_eq!(Spectrum::from(10.0) - Spectrum::from(6.0),
                   Spectrum::from(4.0));
    }

    #[test]
    fn it_can_be_added() {
        let s1 = [(400.0, -3.0), (500.0, -3.0), (600.0, -3.0), (700.0, -3.0)];
        let s2 = [(500.0, 3.0), (700.0, 3.0), (600.0, 3.0), (400.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 Spectrum::from_samples(&s2)).is_black());

        assert_eq!(Spectrum::from(10.0) + Spectrum::from(6.0),
                   Spectrum::from(16.0));
    }

    #[test]
    fn it_can_be_scale() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 (-1.0) * Spectrum::from_samples(&s1)).is_black());

        assert_eq!(Spectrum::from(10.0) * 6.0,
                   Spectrum::from(10.0 * 6.0));
    }

    #[test]
    fn it_can_be_divided_by_scalars() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 (-3.0) * (Spectrum::from_samples(&s1) / 3.0)).is_black());

        assert_eq!(Spectrum::from(10.0) / 6.0,
                   Spectrum::from(10.0 / 6.0));
    }

    #[test]
    fn it_can_be_negated() {
        let s1 = [(400.0, 3.0), (500.0, 3.0), (600.0, 3.0), (700.0, 3.0)];
        assert!((Spectrum::from_samples(&s1) +
                 -Spectrum::from_samples(&s1)).is_black());

        assert_eq!(-Spectrum::from(10.0),
                   Spectrum::from(-10.0));
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

        let s = Spectrum::from(1.0);
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
        assert_eq!(Spectrum::from(1.0)[3], 3.0);
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
        let mut sp = Spectrum::from(1.0);
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

        let s3 = Spectrum::from(10.0);
        let s4 = Spectrum::from(6.0);
        assert_eq!(s3.lerp(&s4, 0.75).coeffs(), [7.0; 3]);
        assert_eq!(s3.lerp(&s4, 0.0).coeffs(), [10.0; 3]);
        assert_eq!(s3.lerp(&s4, 1.0).coeffs(), [6.0; 3]);
    }
}

