use geometry::point::Point;
use geometry::vector::Vector;
use light::Light;
use light::LightSample;
use scene::Scene;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use visibility_tester::VisibilityTester;

use light::internal;

#[derive(Clone, Debug, PartialEq)]
pub struct PointLight {
    base: internal::LightBase,
    light_pos: Point,
    intensity: Spectrum
}

impl PointLight {
    pub fn new(l2w: Transform, intensity: Spectrum) -> PointLight {
        let light_pos = l2w.xf(Point::new());
        PointLight { base: internal::LightBase::new(l2w), light_pos, intensity }
    }
}

impl Light for PointLight {
    fn sample_l(&self, p: &Point, p_eps: f32, ls: LightSample, time: f32)
                -> (Spectrum, Vector, f32, VisibilityTester) {
        let w_i = &self.light_pos - p;
        let pdf = 1.0;
        let vis = VisibilityTester::segment(
            p.clone(), p_eps, self.light_pos.clone(), 0.0, time);
        (self.intensity.clone() / w_i.length_squared(), w_i, pdf, vis)
    }

    fn power(&self, _: &Scene) -> Spectrum {
        ::std::f32::consts::PI * 4.0 * self.intensity
    }

    fn is_delta_light(&self) -> bool { true }
}
