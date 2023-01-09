pub mod exponential;
pub mod volume_grid;

use bbox::HasBounds;
use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use volume::VolumeRegion;

use std::fmt::Debug;
use std::ops::Deref;
use std::marker::Send;
use std::marker::Sync;

use volume::phase_hg;

mod internal {
    use super::*;

    pub trait DensityRegion {
        fn get_sig_a(&self) -> Spectrum;
        fn get_sig_s(&self) -> Spectrum;
        fn get_le(&self) -> Spectrum;
        fn get_g(&self) -> f32;
        fn world_to_volume(&self) -> Transform;

        fn density(&self, p: Point) -> f32;
    }
}

impl<T: internal::DensityRegion +
     Send + Sync + Debug + HasBounds + Intersectable<(f32, f32)>
     > VolumeRegion for T {
    fn sigma_a(&self, p: &Point, _: &Vector, _: f32) -> Spectrum {
        self.get_sig_a() * self.density(self.world_to_volume().t(p))
    }

    fn sigma_s(&self, p: &Point, _: &Vector, _: f32) -> Spectrum {
        self.get_sig_s() * self.density(self.world_to_volume().t(p))
    }

    fn l_ve(&self, p: &Point, _: &Vector, _: f32) -> Spectrum {
        self.get_le() * self.density(self.world_to_volume().t(p))
    }

    fn p(&self, _: &Point, w: &Vector, wp: &Vector, _: f32) -> f32 {
        phase_hg(w, wp, self.get_g())
    }

    fn tau(&self, _: &Ray, _: f32, _: f32) -> Spectrum {
        unimplemented!()
    }
}
