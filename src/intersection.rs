use bsdf::BSDF;
use spectrum::Spectrum;
use vector::Vector;

#[derive(Debug, Copy, Clone)]
pub struct Intersection {
    pub ray_epsilon: f32,
    bsdf: BSDF
}

impl Intersection {
    pub fn new() -> Intersection { Intersection {
        ray_epsilon: 0f32,
        bsdf: BSDF::new()
    } }
    pub fn get_bsdf(&self) -> BSDF { self.bsdf }
    pub fn le(&self, dir: &Vector) -> Spectrum { Spectrum::from_value(0f32) }
}
