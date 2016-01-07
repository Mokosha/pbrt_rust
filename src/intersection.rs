use bsdf::BSDF;
use geometry::vector::Vector;
use ray::Ray;
use spectrum::Spectrum;

#[derive(Debug, Clone)]
pub struct Intersection<'a> {
    pub ray_epsilon: f32,
    bsdf: BSDF<'a>
}

impl<'a> Intersection<'a> {
    pub fn new() -> Intersection<'a> { Intersection {
        ray_epsilon: 0f32,
        bsdf: BSDF::new()
    } }
    pub fn get_bsdf(&self) -> &BSDF<'a> { &self.bsdf }
    pub fn le(&self, dir: &Vector) -> Spectrum { Spectrum::from_value(0f32) }
}

pub trait Intersectable<'a, T = Intersection<'a>> {
    fn intersect(&'a self, ray : &Ray) -> Option<T> { None }
    fn intersect_p(&'a self, ray : &Ray) -> bool { false }
}
