#[derive(Debug, Copy, Clone)]
pub struct Vector;
#[derive(Debug, Copy, Clone)]
pub struct Point;
#[derive(Debug, Copy, Clone)]
pub struct Normal;

impl ::std::ops::Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        self
    }
}

pub fn abs_dot(v: &Vector, n: &Normal) -> f32 { 0f32 }
