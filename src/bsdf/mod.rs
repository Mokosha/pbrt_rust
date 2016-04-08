mod utils;
pub mod bssrdf;
pub mod fresnel;
pub mod specular;
pub mod lambertian;

use bsdf::utils::*;
use diff_geom::DifferentialGeometry;
use geometry::vector::Vector;
use rng::RNG;
use spectrum::Spectrum;

bitflags! {
    pub flags BxDFType: u32 {
        const BSDF_REFLECTION = (1 << 0),
        const BSDF_TRANSMISSION = (1 << 1),
        const BSDF_DIFFUSE = (1 << 2),
        const BSDF_GLOSSY = (1 << 3),
        const BSDF_SPECULAR = (1 << 4),
        const BSDF_ALL_TYPES =
            BSDF_DIFFUSE.bits | BSDF_GLOSSY.bits | BSDF_SPECULAR.bits,
        const BSDF_ALL_REFLECTION =
            BSDF_REFLECTION.bits | BSDF_ALL_TYPES.bits,
        const BSDF_ALL_TRANSMISSION =
            BSDF_TRANSMISSION.bits | BSDF_ALL_TYPES.bits,
        const BSDF_ALL =
            BSDF_ALL_TRANSMISSION.bits | BSDF_ALL_REFLECTION.bits
    }
}

pub trait BxDF {
    fn matches_flags(&self, BxDFType) -> bool;
    fn f(&self, &Vector, &Vector) -> Spectrum;
    fn sample_f(&self, &Vector, f32, f32) -> (Vector, f32, Spectrum);

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

#[derive(Debug, Clone)]
pub struct BSDF {
    pub dg_shading: DifferentialGeometry
}

impl BSDF {
    pub fn new() -> BSDF {
        BSDF {
            dg_shading: DifferentialGeometry::new()
        }
    }

    pub fn sample_bsdf_f(&self, vo: &Vector, sample: BSDFSample,
                         bxdf_type: BxDFType) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}

impl BxDF for BSDF {
    fn matches_flags(&self, ty: BxDFType) -> bool {
        unimplemented!()
    }

    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        unimplemented!()
    }

    fn sample_f(&self, wo: &Vector, u1: f32,
                u2: f32) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
}

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
        let flags = BSDF_REFLECTION | BSDF_TRANSMISSION;
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

pub struct ScaledBxDF<T: BxDF> {
    bxdf: T,
    scale: Spectrum
}

impl<T: BxDF> ScaledBxDF<T> {
    pub fn new(input: T, sc: Spectrum) -> ScaledBxDF<T> {
        ScaledBxDF {
            bxdf: input,
            scale: sc
        }
    }
}

impl<T: BxDF> BxDF for ScaledBxDF<T> {
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

