use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Vector;
use light::Light;
use light::LightSample;
use scene::Scene;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Degrees;
use visibility_tester::VisibilityTester;

use light::internal;

#[derive(Clone, Debug, PartialEq)]
pub struct SpotLight {
    base: internal::LightBase,
    light_pos: Point,
    intensity: Spectrum,
    cos_total_width: f32,
    cos_falloff_start: f32
}

impl SpotLight {
    pub fn new(l2w: Transform, intensity: Spectrum, width: f32, fall: f32)
               -> SpotLight {
        let light_pos = l2w.xf(Point::new());
        SpotLight {
            base: internal::LightBase::new(l2w),
            light_pos,
            intensity,
            cos_total_width: width.as_radians().cos(),
            cos_falloff_start: fall.as_radians().cos(),
        }
    }

    fn falloff(&self, w: Vector) -> f32 {
        let wl = self.base.world_to_light.xf(w);
        let cos_theta = wl.z;
        if cos_theta < self.cos_total_width { 0.0 }
        else if cos_theta > self.cos_falloff_start { 1.0 }
        else {
            let delta = (cos_theta - self.cos_total_width) /
                (self.cos_falloff_start - self.cos_total_width);
            delta * delta * delta * delta
        }
    }
}

impl Light for SpotLight {
    fn sample_l(&self, p: &Point, p_eps: f32, ls: LightSample, time: f32)
                -> (Spectrum, Vector, f32, VisibilityTester) {
        let w_i = (&self.light_pos - p).normalize();
        let pdf = 1.0;
        let vis = VisibilityTester::segment(
            p.clone(), p_eps, self.light_pos.clone(), 0.0, time);
        let i = self.intensity.clone() * self.falloff(-w_i.clone());
        (i / w_i.length_squared(), w_i, pdf, vis)
    }

    fn power(&self, _: &Scene) -> Spectrum {
        let falloff_scale = 1.0 -
            0.5 * (self.cos_falloff_start + self.cos_total_width);
        ::std::f32::consts::PI * 2.0 * falloff_scale * self.intensity
    }

    fn is_delta_light(&self) -> bool { true }
}
