use bsdf;
use bsdf::BxDF;
use bsdf::utils::*;
use geometry::vector::*;
use geometry::normal::Normalize;
use geometry::point::Point;
use spectrum::Spectrum;
use utils::kdtree::*;

fn brdf_remap(wo: &Vector, wi: &Vector) -> Point {
    let cosi = cos_theta(wi);
    let coso = cos_theta(wo);

    let sini = sin_theta(wi);
    let sino = sin_theta(wo);

    let phii = spherical_phi(wi);
    let phio = spherical_phi(wo);

    let dphi = {
        let diff = phii - phio;
        let d = if diff < 0.0 {
            diff + 2.0 * ::std::f32::consts::PI
        } else if diff > (2.0 * ::std::f32::consts::PI) {
            diff - 2.0 * ::std::f32::consts::PI
        } else { diff };

        if d <= ::std::f32::consts::PI { d } else {
            ::std::f32::consts::PI * 2.0 - d
        }
    };

    Point::new_with(sini * sino, dphi / ::std::f32::consts::PI, cosi * coso)
}

#[derive(Clone, Debug, PartialEq)]
pub struct IrregIsotropicSample {
    p: Point,
    v: Spectrum
}

impl IrregIsotropicSample {
    pub fn new(pp: &Point, vv: &Spectrum) -> IrregIsotropicSample {
        IrregIsotropicSample {
            p: pp.clone(),
            v: vv.clone()
        }
    }
}

impl HasPoint for IrregIsotropicSample {
    fn p<'a>(&'a self) -> &'a Point { &(self.p) }
}

#[derive(Debug, Clone)]
pub struct IrregIsotropic {
    iso_data: KdTree<IrregIsotropicSample>
}

impl IrregIsotropic {
    pub fn new(data: KdTree<IrregIsotropicSample>) -> IrregIsotropic {
        IrregIsotropic { iso_data: data }
    }
}

struct IrregIsoProc {
    num_found: usize,
    v: Spectrum,
    sum_weights: f32
}

impl IrregIsoProc {
    fn new() -> IrregIsoProc {
        IrregIsoProc {
            num_found: 0,
            v: Spectrum::from(0.0),
            sum_weights: 0.0
        }
    }
}

impl KdTreeProc<IrregIsotropicSample> for IrregIsoProc {
    fn run(&mut self, p: &Point, sample: &IrregIsotropicSample,
           d2: f32, _: &mut f32) {
        let weight = (-100.0 * d2).exp();
        self.v = self.v + weight * sample.v;
        self.sum_weights += weight;
        self.num_found += 1;
    }
}

impl BxDF for IrregIsotropic {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BxDFType::BSDF_REFLECTION | bsdf::BxDFType::BSDF_GLOSSY).contains(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        let m = brdf_remap(wo, wi);
        let mut last_max_dist_sq: f32 = 0.001;
        loop {
            // Try to find enough BRDF samples around m within search radius
            let mut p = IrregIsoProc::new();
            let max_dist_sq = last_max_dist_sq;
            self.iso_data.lookup(&m, &mut p, max_dist_sq);

            if p.num_found > 2 || last_max_dist_sq > 1.5 {
                return p.v.clamp(1.0, ::std::f32::MAX) / p.sum_weights;
            }

            last_max_dist_sq *= 2.0;
        }
    }

    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct RegularHalfangle {
    num_theta_h: usize,
    num_theta_d: usize,
    num_phi_d: usize,
    brdf: Vec<f32>
}

impl RegularHalfangle {
    pub fn new(nthh: usize, nthd: usize, nphd: usize, d: Vec<f32>) -> RegularHalfangle {
        assert_eq!(nthh * nthd * nphd, d.len());
        RegularHalfangle {
            num_theta_h: nthh,
            num_theta_d: nthd,
            num_phi_d: nphd,
            brdf: d
        }
    }
}

impl BxDF for RegularHalfangle {
    fn matches_flags(&self, ty: bsdf::BxDFType) -> bool {
        (bsdf::BxDFType::BSDF_REFLECTION | bsdf::BxDFType::BSDF_GLOSSY).contains(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        // Compute w_h and transform w_i to halfangle coordinate system
        let wh = (wi + wo).normalize();
        let wh_theta = spherical_theta(&wh);
        let (wh_cos_phi, wh_sin_phi) = (cos_phi(&wh), sin_phi(&wh));
        let (wh_cos_theta, wh_sin_theta) = (cos_theta(&wh), sin_theta(&wh));
        let whx = Vector::new_with(wh_cos_phi * wh_cos_theta,
                                   wh_sin_phi * wh_cos_theta,
                                   -wh_sin_theta);

        let why = Vector::new_with(-wh_sin_phi, wh_cos_phi, 0.0);
        let wd = Vector::new_with(wi.dot(&whx), wi.dot(&why), wi.dot(&wh));

        // Compute index into measured BRDF tables
        let (wd_theta, wd_phi) = {
            let (t, p) = (spherical_theta(&wd), spherical_phi(&wd));
            if p > ::std::f32::consts::PI {
                (t, p - ::std::f32::consts::PI)
            } else {
                (t, p)
            }
        };

        // Compute wh_theta_index, wd_theta_index, and wd_phi_index
        let remap = |v: f32, mx: f32, cnt: usize| { (((v / mx) * (cnt as f32)) as usize).clamp(0, cnt - 1) };
        let wh_theta_index = remap((wh_theta / (::std::f32::consts::PI / 2.0)).max(0.0).sqrt(),
                                   1.0, self.num_theta_h);
        let wd_theta_index = remap(wd_theta, ::std::f32::consts::PI / 2.0, self.num_theta_d);
        let wd_phi_index = remap(wd_phi, ::std::f32::consts::PI, self.num_phi_d);

        let index = wd_phi_index + self.num_phi_d *
            (wd_theta_index + wh_theta_index * self.num_theta_d);

        let rgb = [self.brdf[index * 3 + 0],
                   self.brdf[index * 3 + 1],
                   self.brdf[index * 3 + 2]];
        Spectrum::from_rgb(rgb)
    }

    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}
