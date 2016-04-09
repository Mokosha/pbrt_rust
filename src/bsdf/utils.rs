use geometry::vector::Vector;
use utils::Clamp;

use std::f32;

pub fn cos_theta(v: &Vector) -> f32 { v.z }
pub fn abs_cos_theta(v: &Vector) -> f32 { v.z.abs() }
pub fn sin_theta2(v: &Vector) -> f32 { 0f32.max(1.0 - v.z*v.z) }
pub fn sin_theta(v: &Vector) -> f32 { sin_theta2(v).sqrt() }

pub fn cos_phi(v: &Vector) -> f32 {
    let vx = v.x;
    let sintheta = sin_theta(v);
    if sintheta == 0.0 {
        1.0
    } else {
        (vx / sintheta).clamp(-1.0, 1.0)
    }
}

pub fn sin_phi(v: &Vector) -> f32 {
    let vy = v.y;
    let sintheta = sin_theta(v);
    if sintheta == 0.0 {
        0.0
    } else {
        (vy / sintheta).clamp(-1.0, 1.0)
    }
}
