pub struct Vector;
pub struct Point;
pub struct Normal;

impl ::std::ops::Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        self
    }
}
