use bsdf::BSDF;
use spectrum::Spectrum;
use vector::Vector;

#[derive(Debug, Copy, Clone)]
pub struct Intersection;

impl Intersection {
    pub fn get_bsdf(&self) -> BSDF { BSDF::new() }
    pub fn le(&self, dir: &Vector) -> Spectrum { Spectrum::from_value(0f32) }
}
