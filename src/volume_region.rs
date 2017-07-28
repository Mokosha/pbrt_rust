use bbox::BBox;
use bbox::HasBounds;
use geometry::point::Point;
use geometry::vector::Dot;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use spectrum::Spectrum;

use std::fmt::Debug;
use std::ops::Deref;
use std::marker::Send;
use std::marker::Sync;

use ::std::f32::consts::PI;

mod internal {
    use super::*;

    pub trait VolumeRegionBase {
        fn sigma_a(&self, &Point, &Vector, f32) -> Spectrum;
        fn sigma_s(&self, &Point, &Vector, f32) -> Spectrum;
        fn l_ve(&self, &Point, &Vector, f32) -> Spectrum;
        fn p(&self, &Point, &Vector, &Vector, f32) -> f32;
        fn tau(&self, &Ray, f32, f32) -> Spectrum;
    }

    impl<T> VolumeRegionBase for T where T: Deref<Target = VolumeRegion> {
        fn sigma_a(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            VolumeRegionBase::sigma_a(self.deref(), p, w, time)
        }

        fn sigma_s(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            VolumeRegionBase::sigma_s(self.deref(), p, w, time)
        }

        fn l_ve(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            VolumeRegionBase::l_ve(self.deref(), p, w, time)
        }

        fn p(&self, pt: &Point, wo: &Vector, wi: &Vector, time: f32) -> f32 {
            VolumeRegionBase::p(self.deref(), pt, wo, wi, time)
        }

        fn tau(&self, ray: &Ray, step: f32, offset: f32) -> Spectrum {
            VolumeRegionBase::tau(self.deref(), ray, step, offset)
        }
    }
}

pub trait VolumeRegion: Send + Sync + Debug + HasBounds
    + Intersectable<(f32, f32)> + internal::VolumeRegionBase {
        fn sigma_a(&self, &Point, &Vector, f32) -> Spectrum;
        fn sigma_s(&self, &Point, &Vector, f32) -> Spectrum;
        fn l_ve(&self, &Point, &Vector, f32) -> Spectrum;
        fn p(&self, &Point, &Vector, &Vector, f32) -> f32;
        fn tau(&self, &Ray, f32, f32) -> Spectrum;

        fn sigma_t(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            internal::VolumeRegionBase::sigma_a(self, p, w, time) +
                internal::VolumeRegionBase::sigma_s(self, p, w, time)
        }
}

impl<T> VolumeRegion for T where T:
Send + Sync + Debug + HasBounds
    + Intersectable<(f32, f32)> + internal::VolumeRegionBase {
        fn sigma_a(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            (self as &internal::VolumeRegionBase).sigma_a(p, w, time)
        }

        fn sigma_s(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            (self as &internal::VolumeRegionBase).sigma_s(p, w, time)
        }

        fn l_ve(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            (self as &internal::VolumeRegionBase).l_ve(p, w, time)
        }

        fn p(&self, pt: &Point, wo: &Vector, wi: &Vector, time: f32) -> f32 {
            (self as &internal::VolumeRegionBase).p(pt, wo, wi, time)
        }

        fn tau(&self, ray: &Ray, step: f32, offset: f32) -> Spectrum {
            (self as &internal::VolumeRegionBase).tau(ray, step, offset)
        }
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
