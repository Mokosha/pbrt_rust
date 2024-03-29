mod utils;
pub mod bssrdf;
pub mod fresnel;
pub mod lambertian;
pub mod measured;
pub mod microfacet;
pub mod orennayar;
pub mod specular;

use bsdf::utils::*;
use diff_geom::DifferentialGeometry;
use geometry::vector::*;
use geometry::normal::*;
use rng::RNG;
use spectrum::Spectrum;

use std::clone::Clone;
use std::fmt::Debug;
use std::marker::Sized;

bitflags! {
    pub struct BxDFType: u32 {
        const BSDF_REFLECTION = (1 << 0);
        const BSDF_TRANSMISSION = (1 << 1);
        const BSDF_DIFFUSE = (1 << 2);
        const BSDF_GLOSSY = (1 << 3);
        const BSDF_SPECULAR = (1 << 4);
        const BSDF_ALL_TYPES =
            Self::BSDF_DIFFUSE.bits | Self::BSDF_GLOSSY.bits | Self::BSDF_SPECULAR.bits;
        const BSDF_ALL_REFLECTION =
            Self::BSDF_REFLECTION.bits | Self::BSDF_ALL_TYPES.bits;
        const BSDF_ALL_TRANSMISSION =
            Self::BSDF_TRANSMISSION.bits | Self::BSDF_ALL_TYPES.bits;
        const BSDF_ALL =
            Self::BSDF_ALL_TRANSMISSION.bits | Self::BSDF_ALL_REFLECTION.bits;
    }
}

pub trait BxDF : Debug + 'static {
    fn matches_flags(&self, _: BxDFType) -> bool;
    fn f(&self, _: &Vector, _: &Vector) -> Spectrum;
    fn sample_f(&self, _: &Vector, _: f32, _: f32) -> (Vector, f32, Spectrum);

    fn rho_hd(&self, v: &Vector, samples: &[f32]) -> Spectrum {
        unimplemented!()
    }

    fn rho_hh(&self, samples1: &[f32], samples2: &[f32]) -> Spectrum {
        unimplemented!()
    }
}

pub struct BSDFSample;
impl BSDFSample {
    pub fn new(rng: &mut RNG) -> BSDFSample { BSDFSample }
}

#[derive(Debug)]
pub struct BSDF {
    pub dg_shading: DifferentialGeometry,
    pub eta: f32,
    nn: Normal,
    ng: Normal,
    sn: Vector,
    tn: Vector,
    bxdfs: Vec<Box<dyn BxDF>>
}

impl BSDF {
    pub fn new_with_eta(dg: DifferentialGeometry, n_geom: Normal, e: f32) -> BSDF {
        let shading_normal = dg.nn.clone();
        let shading_normal_t = dg.dpdu.clone().normalize();
        let shading_normal_s =
            Vector::from(shading_normal.clone())
            .into_cross_with(&shading_normal_t);

        BSDF {
            dg_shading: dg,
            eta: e,
            nn: shading_normal,
            ng: n_geom,
            sn: shading_normal_s,
            tn: shading_normal_t,
            bxdfs: Vec::with_capacity(8)
        }
    }

    pub fn new(dg: DifferentialGeometry, n_geom: Normal) -> BSDF {
        BSDF::new_with_eta(dg, n_geom, 1.0)
    }

    pub fn add_bxdf<T: BxDF>(&mut self, bxdf: T) {
        self.bxdfs.push(Box::new(bxdf));
    }

    pub fn num_components(&self) -> usize { self.bxdfs.len() }
    pub fn num_components_matching(&self, flags: BxDFType) -> usize {
        self.bxdfs.iter().fold(0, |acc, bxdf| {
            if bxdf.matches_flags(flags) {
                acc + 1
            } else {
                acc
            }
        })
    }

    pub fn world_to_local(&self, v: Vector) -> Vector {
        Vector::new_with(v.dot(&self.sn), v.dot(&self.tn), v.dot(&self.nn))
    }

    pub fn local_to_world(&self, v: Vector) -> Vector {
        Vector::new_with(self.sn.x * v.x + self.tn.x * v.y + self.nn.x * v.z,
                         self.sn.y * v.x + self.tn.y * v.y + self.nn.y * v.z,
                         self.sn.z * v.x + self.tn.z * v.y + self.nn.z * v.z)
    }

    pub fn mix_with(self, other: BSDF, s: Spectrum) -> BSDF {
        let s2 = (Spectrum::from(1.0) - s.clone()).clamp(0.0, ::std::f32::MAX);

        let mut ret = BSDF::new_with_eta(self.dg_shading.clone(), self.ng.clone(), self.eta);
        for b in self.bxdfs.into_iter() {
            ret.add_bxdf(ScaledBxDF::new(b, s.clone()));
        }

        for b in other.bxdfs.into_iter() {
            ret.add_bxdf(ScaledBxDF::new(b, s2.clone()));
        }

        ret
    }

    pub fn f(&self, wo_w: Vector, wi_w: Vector, in_flags: BxDFType) -> Spectrum {
        let flags = if wi_w.dot(&self.ng) * wo_w.dot(&self.ng) > 0.0 {
            in_flags & !BxDFType::BSDF_TRANSMISSION
        } else {
            in_flags & !BxDFType::BSDF_REFLECTION
        };

        let wo = self.world_to_local(wo_w);
        let wi = self.world_to_local(wi_w);

        self.bxdfs.iter().fold(Spectrum::from(0.0), |f, bxdf| {
            if bxdf.matches_flags(flags) {
                f + bxdf.f(&wo, &wi)
            } else {
                f
            }
        })
    }

    pub fn sample_f(&self, vo: &Vector, sample: BSDFSample,
                    bxdf_type: BxDFType) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct BRDFtoBTDF<T: BxDF> {
    brdf: T
}

impl<T: BxDF> BRDFtoBTDF<T> {
    pub fn new(input: T) -> BRDFtoBTDF<T> { BRDFtoBTDF { brdf: input } }
}

fn other_hemi(v: &Vector) -> Vector {
    Vector::new_with(v.x, v.y, -v.z)
}

impl<T: BxDF> BxDF for BRDFtoBTDF<T> {
    fn matches_flags(&self, ty: BxDFType) -> bool {
        let flags = BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION;
        self.brdf.matches_flags(ty ^ flags)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        self.brdf.f(wo, &other_hemi(wi))
    }

    fn sample_f(&self, wo: &Vector, u1: f32,
                u2: f32) -> (Vector, f32, Spectrum) {
        let (wi, pdf, v) = self.brdf.sample_f(wo, u1, u2);
        (other_hemi(&wi), pdf, v)
    }

    fn rho_hd(&self, v: &Vector, samples: &[f32]) -> Spectrum {
        self.brdf.rho_hd(v, samples)
    }

    fn rho_hh(&self, samples1: &[f32], samples2: &[f32]) -> Spectrum {
        self.brdf.rho_hh(samples1, samples2)
    }
}

#[derive(Debug)]
pub struct ScaledBxDF {
    bxdf: Box<dyn BxDF>,
    scale: Spectrum
}

impl ScaledBxDF {
    pub fn new(input: Box<dyn BxDF>, sc: Spectrum) -> ScaledBxDF {
        ScaledBxDF {
            bxdf: input,
            scale: sc
        }
    }
}

impl BxDF for ScaledBxDF {
    fn matches_flags(&self, ty: BxDFType) -> bool {
        self.bxdf.matches_flags(ty)
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        self.bxdf.f(wo, wi) * self.scale
    }

    fn sample_f(&self, wo: &Vector, u1: f32,
                u2: f32) -> (Vector, f32, Spectrum) {
        let (wi, pdf, v) = self.bxdf.sample_f(wo, u1, u2);
        (wi, pdf, self.scale * v)
    }

    fn rho_hd(&self, v: &Vector, samples: &[f32]) -> Spectrum {
        self.bxdf.rho_hd(v, samples) * self.scale
    }

    fn rho_hh(&self, samples1: &[f32], samples2: &[f32]) -> Spectrum {
        self.bxdf.rho_hh(samples1, samples2) * self.scale
    }
}
