use geometry::vector::Vector;
use utils::Clamp;

use std::f32;

fn cos_theta(v: Vector) -> f32 { v.z }
fn abs_cos_theta(v: Vector) -> f32 { v.z.abs() }
fn sin_theta2(v: Vector) -> f32 { 0f32.max(1.0 - v.z*v.z) }
fn sin_theta(v: Vector) -> f32 { sin_theta2(v).sqrt() }

fn cos_phi(v: Vector) -> f32 {
    let vx = v.x;
    let sintheta = sin_theta(v);
    if sintheta == 0.0 {
        1.0
    } else {
        (vx / sintheta).clamp(-1.0, 1.0)
    }
}

fn sin_phi(v: Vector) -> f32 {
    let vy = v.y;
    let sintheta = sin_theta(v);
    if sintheta == 0.0 {
        0.0
    } else {
        (vy / sintheta).clamp(-1.0, 1.0)
    }
}
