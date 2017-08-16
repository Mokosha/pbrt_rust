pub mod point;
pub mod spot;

use ray::RayDifferential;
use rng::RNG;
use spectrum::Spectrum;
use visibility_tester::VisibilityTester;
use geometry::point::Point;
use geometry::vector::Vector;
use scene::Scene;
use transform::transform::Transform;

mod internal {
    use super::*;

    #[derive(Clone, Debug, PartialOrd, PartialEq)]
    pub struct LightBase {
        pub num_samples: usize,
        pub light_to_world: Transform,
        pub world_to_light: Transform
    }

    impl LightBase {
        pub fn new(l2w: Transform) -> LightBase {
            LightBase {
                num_samples: 1,
                light_to_world: l2w.clone(),
                world_to_light: l2w.invert()
            }
        }

        pub fn new_with_samples(l2w: Transform, ns: usize) -> LightBase {
            LightBase {
                num_samples: ns,
                light_to_world: l2w.clone(),
                world_to_light: l2w.invert()
            }
        }
    }
}

#[derive(PartialOrd,Ord,PartialEq,Eq)]
pub struct LightSample;

impl LightSample {
    pub fn new(rng: &mut RNG) -> LightSample { LightSample }
}

pub trait Light : ::std::marker::Send + ::std::marker::Sync {
    fn le(&self, &RayDifferential) -> Spectrum {
        Spectrum::from(0.0)
    }

    fn sample_l(&self, &Point, f32, LightSample, f32)
                -> (Spectrum, Vector, f32, VisibilityTester);
    fn power(&self, &Scene) -> Spectrum;
    fn is_delta_light(&self) -> bool;
}
