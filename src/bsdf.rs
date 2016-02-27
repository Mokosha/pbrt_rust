use diff_geom::DifferentialGeometry;
use geometry::vector::Vector;
use rng::RNG;
use spectrum::Spectrum;

pub const BSDF_SPECULAR : u32 = (1 << 0);
pub const BSDF_REFLECTION : u32 = (1 << 1);
pub const BSDF_TRANSMISSION : u32 = (1 << 2);

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

    pub fn f(&self, vo: &Vector, vi: &Vector) -> Spectrum {
        Spectrum::from_value(0f32)
    }

    pub fn sample_f(&self, vo: &Vector, sample: BSDFSample, bxdf_type: u32) ->
        (Vector, f32, Spectrum) {
            (Vector::new(), 0f32, Spectrum::from_value(0f32))
        }
}

pub struct BSSRDF;

#[cfg(test)]
mod tests {
    use super::*;
}
