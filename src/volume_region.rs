use bbox::BBox;
use bbox::HasBounds;
use geometry::vector::Dot;
use geometry::vector::Vector;

use std::fmt::Debug;
use std::ops::Deref;
use std::marker::Send;
use std::marker::Sync;

use ::std::f32::consts::PI;

mod internal {
    use super::*;

    pub trait VolumeRegionBase {
        fn world_bound(&self) -> BBox;
    }

    impl<T> VolumeRegionBase for T where T: Deref<Target = VolumeRegion> {
        fn world_bound(&self) -> BBox { self.deref().world_bound() }
    }
}

pub trait VolumeRegion: Send + Sync {
    fn world_bound(&self) -> BBox;
}

impl<T> VolumeRegion for T
where T : Send + Sync + Debug + Clone + internal::VolumeRegionBase {
    fn world_bound(&self) -> BBox {
        (self as &internal::VolumeRegionBase).world_bound()
    }
}

impl<T : VolumeRegion> HasBounds for T {
    fn world_bound(&self) -> BBox { self.world_bound() }
}

pub fn phase_isotropic(_: &Vector, _: &Vector) -> f32 {
    1.0 / (4.0 * PI)
}

pub fn phase_rayleigh(w: &Vector, wp: &Vector) -> f32 {
    let cos_theta = w.dot(wp);
    3.0 / (16.0 * PI) * (1.0 + cos_theta * cos_theta)
}

pub fn phase_mie_hazy(w: &Vector, wp: &Vector) -> f32 {
    let cos_theta = w.dot(wp);
    (0.5 + 4.5 * (0.5 * (1.0 + cos_theta)).powi(8)) / (4.0 * PI)
}

pub fn phase_mie_murky(w: &Vector, wp: &Vector) -> f32 {
    let cos_theta = w.dot(wp);
    (0.5 + 16.5 * (0.5 * (1.0 + cos_theta)).powi(32)) / (4.0 * PI)
}

pub fn phase_hg(w: &Vector, wp: &Vector, g: f32) -> f32 {
    let cos_theta = w.dot(wp);
    let gsq = g * g;
    let factor = 1.0 / (4.0 * PI);
    factor * (1.0 - gsq) / (1.0 + gsq - 2.0 * g * cos_theta).powf(1.5)
}

pub fn phase_schlick(w: &Vector, wp: &Vector, g: f32) -> f32 {
    let alpha = 1.5;
    let k = (1.0 - alpha) * (g * g * g) + alpha * g;
    let k_cos_theta = k * w.dot(wp);
    1.0 / (4.0 * PI) * (1.0 - k * k) / ((1.0 - k_cos_theta).powi(2))
}
