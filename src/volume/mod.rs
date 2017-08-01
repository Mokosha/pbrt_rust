pub mod homogeneous;

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

pub trait VolumeRegion:
Send + Sync + Debug + HasBounds + Intersectable<(f32, f32)> {
        fn sigma_a(&self, &Point, &Vector, f32) -> Spectrum;
        fn sigma_s(&self, &Point, &Vector, f32) -> Spectrum;
        fn l_ve(&self, &Point, &Vector, f32) -> Spectrum;
        fn p(&self, &Point, &Vector, &Vector, f32) -> f32;
        fn tau(&self, &Ray, f32, f32) -> Spectrum;

        fn sigma_t(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
            self.sigma_a(p, w, time) + self.sigma_s(p, w, time)
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
