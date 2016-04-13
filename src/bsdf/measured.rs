use bsdf;
use bsdf::BxDF;
use bsdf::utils::*;
use geometry::vector::*;
use geometry::point::Point;
use spectrum::Spectrum;
use utils::Clamp;

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

struct IrregIsotropicSample {
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

// !FIXME! Needs implementation!
struct KdTree<T> {
    data: Vec<T>
}

trait KdTreeProc<T> {
    fn run(&mut self, &Point, &T, f32, &mut f32);
}

impl<T> KdTree<T> {
    fn lookup<U: KdTreeProc<T>>(&self, m: &Point, p: &mut U, max_dist_sq: f32) {
        // Actually need a KdTree here...
        unimplemented!()
    }
}

pub struct IrregIsotropic {
    iso_data: KdTree<IrregIsotropicSample>
}

impl IrregIsotropic {
    fn new(data: &KdTree<IrregIsotropicSample>) -> IrregIsotropic {
        // Actually need a KdTree here...
        unimplemented!()
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
        (bsdf::BSDF_REFLECTION | bsdf::BSDF_GLOSSY).contains(ty)
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
