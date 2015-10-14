use vector::Vector;

pub struct Ray;
impl Ray {
    pub fn depth(&self) -> i32 { 0 }
    pub fn dir(self) -> Vector { Vector }
}

pub struct RayDifferential;

impl RayDifferential {
    pub fn scale_differentials(&mut self, s: f32) { }
    pub fn base_ray(&self) -> Ray { Ray }
    pub fn dir(&self) -> Vector { self.base_ray().dir() }
}
