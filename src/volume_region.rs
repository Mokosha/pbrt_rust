use bbox;
use geometry::vector::Dot;
use geometry::vector::Vector;

use ::std::f32::consts::PI;

#[derive(Debug, Copy, Clone)]
pub struct VolumeRegion;

impl bbox::HasBounds for VolumeRegion {
    fn world_bound(&self) -> bbox::BBox { bbox::BBox::new() }
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
