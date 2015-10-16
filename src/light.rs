use ray::RayDifferential;
use rng::RNG;
use spectrum::Spectrum;
use time::Time;
use visibility_tester::VisibilityTester;
use geometry::Point;
use geometry::Vector;

#[derive(PartialOrd,Ord,PartialEq,Eq)]
pub struct Light;
pub struct LightSample;

impl LightSample {
    pub fn new<T: RNG>(rng: &mut T) -> LightSample { LightSample }
}

impl Light {
    pub fn le(&self, ray: &RayDifferential) -> Spectrum {
        Spectrum::from_value(0f32)
    }

    pub fn sample_l(&self, p: &Point, eps: f32,
                    sample: LightSample, time: Time) ->
        (Spectrum, Vector, f32, VisibilityTester) {
            (Spectrum::from_value(0f32), Vector::new(), 0f32, VisibilityTester)
        }
}
