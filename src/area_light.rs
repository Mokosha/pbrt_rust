use light::{Light, LightSample};
use geometry::point::Point;
use geometry::vector::Vector;
use spectrum::Spectrum;
use scene::Scene;
use visibility_tester::VisibilityTester;

#[derive(Clone, PartialEq, Debug)]
pub struct AreaLight;

impl Light for AreaLight {
    fn sample_l(&self, _p: &Point, _f: f32, _ls: LightSample, _ff: f32)
                -> (Spectrum, Vector, f32, VisibilityTester) {
        unimplemented!()
    }

    fn power(&self, _s: &Scene) -> Spectrum {
        unimplemented!()
    }

    fn is_delta_light(&self) -> bool {
        unimplemented!()
    }
}
