use diff_geom::DifferentialGeometry;
use geometry::vector::Vector;
use rng::RNG;
use spectrum::Spectrum;
use utils::Clamp;

use std::f32;

fn cos_theta(v: Vector) -> f32 { v.z }
fn abs_cos_theta(v: Vector) -> f32 { v.z.abs() }
fn sin_theta2(v: Vector) -> f32 { 0f32.max(1.0 - v.z*v.z) }
fn sin_theta(v: Vector) -> f32 { sin_theta2(v).sqrt() }

fn cos_phi(v: Vector) -> f32 {
    let vx = v.x;
    let sintheta = sin_theta(v);
    if sintheta == 0.0 {
        1.0
    } else {
        (vx / sintheta).clamp(-1.0, 1.0)
    }
}

fn sin_phi(v: Vector) -> f32 {
    let vy = v.y;
    let sintheta = sin_theta(v);
    if sintheta == 0.0 {
        0.0
    } else {
        (vy / sintheta).clamp(-1.0, 1.0)
    }
}

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

    fn rho_hd(&self, &Vector, &[f32]) -> Spectrum;
    fn rho_hh(&self, &[f32], &[f32]) -> Spectrum;
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

    pub fn sample_f(&self, vo: &Vector, sample: BSDFSample,
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

    fn rho_hd(&self, v: &Vector, samples: &[f32]) -> Spectrum {
        unimplemented!()
    }

    fn rho_hh(&self, samples1: &[f32], samples2: &[f32]) -> Spectrum {
        unimplemented!()
    }
}

pub struct BRDFtoBTDF<T: BxDF> {
    brdf: T
}

impl<T: BxDF> BRDFtoBTDF<T> {
    pub fn new(input: T) -> BRDFtoBTDF<T> { BRDFtoBTDF { brdf: input } }


    pub fn sample_f(&self, vo: &Vector, sample: BSDFSample,
                    bxdf_type: BxDFType) -> (Vector, f32, Spectrum) {
        unimplemented!()
    }
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

    fn rho_hd(&self, v: &Vector, samples: &[f32]) -> Spectrum {
        self.brdf.rho_hd(v, samples)
    }

    fn rho_hh(&self, samples1: &[f32], samples2: &[f32]) -> Spectrum {
        self.brdf.rho_hh(samples1, samples2)
    }
}

pub struct BSSRDF;

#[cfg(test)]
mod tests {
    use super::*;
}
