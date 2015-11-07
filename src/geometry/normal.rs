use geometry::vector::Vector;
use geometry::vector::Dot;

pub trait Normalize {
    fn normalize(self) -> Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Normal {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Normal {
    pub fn new() -> Normal { Normal { x: 0f32, y: 0f32, z: 0f32 } }
    pub fn new_with(x_: f32, y_: f32, z_: f32) -> Normal {
        Normal { x: x_, y: y_, z: z_ }
    }
}

impl ::std::convert::From<Vector> for Normal {
    fn from(v: Vector) -> Normal {
        Normal::new_with(v.x, v.y, v.z)
    }
}

impl<'a> ::std::convert::From<&'a Vector> for Normal {
    fn from(v: &'a Vector) -> Normal {
        Normal::new_with(v.x, v.y, v.z)        
    }
}

impl ::std::convert::From<Normal> for Vector {
    fn from(v: Normal) -> Vector {
        Vector::new_with(v.x, v.y, v.z)
    }
}

impl<'a> ::std::convert::From<&'a Normal> for Vector {
    fn from(v: &'a Normal) -> Vector {
        Vector::new_with(v.x, v.y, v.z)        
    }
}

impl ::std::ops::Sub for Normal {
    type Output = Normal;
    fn sub(self, _rhs: Normal) -> Normal {
        Normal::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl<'a, 'b> ::std::ops::Sub<&'b Normal> for &'a Normal {
    type Output = Normal;
    fn sub(self, _rhs: &'b Normal) -> Normal {
        Normal::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl ::std::ops::Add for Normal {
    type Output = Normal;
    fn add(self, _rhs: Normal) -> Normal {
        Normal::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a, 'b> ::std::ops::Add<&'b Normal> for &'a Normal {
    type Output = Normal;
    fn add(self, _rhs: &'b Normal) -> Normal {
        Normal::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl ::std::ops::Mul<f32> for Normal {
    type Output = Normal;
    fn mul(self, f: f32) -> Normal {
        Normal::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl<'a> ::std::ops::Mul<f32> for &'a Normal {
    type Output = Normal;
    fn mul(self, f: f32) -> Normal {
        Normal::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl ::std::ops::Mul<Normal> for f32 {
    type Output = Normal;
    fn mul(self, v: Normal) -> Normal { v * self }
}

impl<'a> ::std::ops::Mul<&'a Normal> for f32 {
    type Output = Normal;
    fn mul(self, v: &'a Normal) -> Normal { v * self }
}

impl ::std::ops::Div<f32> for Normal {
    type Output = Normal;
    fn div(self, f: f32) -> Normal {
        let recip = 1f32 / f;
        recip * self
    }
}

impl<'a> ::std::ops::Div<f32> for &'a Normal {
    type Output = Normal;
    fn div(self, f: f32) -> Normal {
        let recip = 1f32 / f;
        recip * self
    }
}

impl ::std::ops::Neg for Normal {
    type Output = Normal;
    fn neg(self) -> Normal {
        Normal::new_with(-self.x, -self.y, -self.z)
    }
}

impl<'a> ::std::ops::Neg for &'a Normal {
    type Output = Normal;
    fn neg(self) -> Normal {
        Normal::new_with(-self.x, -self.y, -self.z)
    }
}

impl ::std::ops::Index<i32> for Normal {
    type Output = f32;
    fn index<'a>(&'a self, index: i32) -> &'a f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Error - Normal index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<i32> for Normal {
    fn index_mut<'a>(&'a mut self, index: i32) -> &'a mut f32 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Error - Normal index out of bounds!")
        }
    }
}

impl Dot for Normal {
    fn dot(&self, n: &Normal) -> f32 {
        self.x * n.x + self.y * n.y + self.z * n.z
    }
}

impl Dot<Vector> for Normal {
    fn dot(&self, v: &Vector) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}

impl Dot<Normal> for Vector {
    fn dot(&self, n: &Normal) -> f32 {
        self.x * n.x + self.y * n.y + self.z * n.z
    }
}

impl Normalize for Normal {
    fn normalize(self) -> Normal {
        let v = Vector::from(self);
        Normal::from(v.normalize())
    }
}

pub fn face_forward(n: &Normal, v: &Vector) -> Normal {
    if n.dot(v) < 0f32 { -n } else { n.clone() }
}
