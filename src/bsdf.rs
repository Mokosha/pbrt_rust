use diff_geom::DifferentialGeometry;
use geometry::Normal;
use geometry::Point;
use geometry::Vector;
use rng::RNG;
use spectrum::Spectrum;

pub const BSDF_SPECULAR : u32 = (1 << 0);
pub const BSDF_REFLECTION : u32 = (1 << 1);
pub const BSDF_TRANSMISSION : u32 = (1 << 2);

pub struct BSDFSample;
impl BSDFSample {
    pub fn new<T: RNG>(rng: &mut T) -> BSDFSample { BSDFSample }
}

#[derive(Debug, Clone)]
pub struct BSDF {
    pub dg_shading: DifferentialGeometry
}

impl BSDF {
    pub fn new() -> BSDF {
        BSDF {
            dg_shading: DifferentialGeometry {
                p: Point::new(),
                nn: Normal::new()
            }
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
