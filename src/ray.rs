use time::Time;
use vector::Vector;

#[derive(Debug, Clone)]
pub struct Ray;
impl Ray {
    pub fn depth(&self) -> i32 { 0 }
    pub fn dir(self) -> Vector { Vector }
}

#[derive(Debug, Clone)]
pub struct RayDifferential;

impl RayDifferential {
    pub fn scale_differentials(&mut self, s: f32) { }
    pub fn base_ray(&self) -> Ray { Ray }
    pub fn dir(&self) -> Vector { self.base_ray().dir() }
    pub fn time(&self) -> Time { Time }
}
