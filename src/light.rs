use ray::RayDifferential;
use spectrum::Spectrum;

#[derive(PartialOrd,Ord,PartialEq,Eq)]
pub struct Light;

impl Light {
    pub fn le(&self, ray: &RayDifferential) -> Spectrum {
        Spectrum::from_value(0f32)
    }
}
