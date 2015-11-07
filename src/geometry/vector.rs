use utils::Lerp;
use geometry::normal::Normalize;

pub trait Dot<T = Self> {
    fn dot(&self, v2: &T) -> f32;
    fn abs_dot(&self, b: &T) -> f32 { self.dot(b).abs() }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector {
    pub fn new() -> Vector { Vector { x: 0f32, y: 0f32, z: 0f32 } }
    pub fn new_with(x_: f32, y_: f32, z_: f32) -> Vector {
        Vector { x: x_, y: y_, z: z_ }
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn cross(self, v2: &Vector) -> Vector {
        Vector::new_with(
            (self.y * v2.z) - (self.z * v2.y),
            (self.z * v2.x) - (self.x * v2.z),
            (self.x * v2.y) - (self.y * v2.x))
    }
}

impl ::std::ops::Sub for Vector {
    type Output = Vector;
    fn sub(self, _rhs: Vector) -> Vector {
        Vector::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl<'a, 'b> ::std::ops::Sub<&'b Vector> for &'a Vector {
    type Output = Vector;
    fn sub(self, _rhs: &'b Vector) -> Vector {
        Vector::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl ::std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, _rhs: Vector) -> Vector {
        Vector::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a> ::std::ops::Add<Vector> for &'a Vector {
    type Output = Vector;
    fn add(self, _rhs: Vector) -> Vector {
        Vector::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a> ::std::ops::Add<&'a Vector> for Vector {
    type Output = Vector;
    fn add(self, _rhs: &'a Vector) -> Vector {
        Vector::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a, 'b> ::std::ops::Add<&'b Vector> for &'a Vector {
    type Output = Vector;
    fn add(self, _rhs: &'b Vector) -> Vector {
        Vector::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl ::std::ops::Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, f: f32) -> Vector {
        Vector::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl<'a> ::std::ops::Mul<f32> for &'a Vector {
    type Output = Vector;
    fn mul(self, f: f32) -> Vector {
        Vector::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl ::std::ops::Mul<Vector> for f32 {
    type Output = Vector;
    fn mul(self, v: Vector) -> Vector { v * self }
}

impl<'a> ::std::ops::Mul<&'a Vector> for f32 {
    type Output = Vector;
    fn mul(self, v: &'a Vector) -> Vector { v * self }
}

impl ::std::ops::Div<f32> for Vector {
    type Output = Vector;
    fn div(self, f: f32) -> Vector {
        let recip = 1f32 / f;
        recip * self
    }
}

impl<'a> ::std::ops::Div<f32> for &'a Vector {
    type Output = Vector;
    fn div(self, f: f32) -> Vector {
        let recip = 1f32 / f;
        recip * self
    }
}

impl ::std::ops::Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::new_with(-self.x, -self.y, -self.z)
    }
}

impl<'a> ::std::ops::Neg for &'a Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::new_with(-self.x, -self.y, -self.z)
    }
}

impl ::std::ops::Index<i32> for Vector {
    type Output = f32;
    fn index<'a>(&'a self, index: i32) -> &'a f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Error - Vector index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<i32> for Vector {
    fn index_mut<'a>(&'a mut self, index: i32) -> &'a mut f32 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Error - Vector index out of bounds!")
        }
    }
}

impl Dot for Vector {
    fn dot(&self, v: &Vector) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}

impl Normalize for Vector {
    fn normalize(self) -> Vector {
        let l = self.length();
        self / l
    }
}

impl Lerp<f32> for Vector {
    fn lerp(&self, b: &Vector, t: f32) -> Vector {
        (1f32 - t) * self + t * b
    }
}

pub fn coordinate_system(v1: &Vector) -> (Vector, Vector) {
    let v2 =
        if (v1.x.abs() > v1.y.abs()) {
            let inv_len = 1f32 / ((v1.x * v1.x + v1.z * v1.z).sqrt());
            Vector::new_with(-v1.x * inv_len, 0f32, v1.x * inv_len)
        } else {
            let inv_len = 1f32 / ((v1.y * v1.y + v1.z * v1.z).sqrt());
            Vector::new_with(0f32, v1.z * inv_len, -v1.y * inv_len)
        };
    let v3 = v1.clone().cross(&v2);
    (v2, v3)
}
