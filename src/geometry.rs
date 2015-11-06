use utils::Lerp;

pub trait Dot<T = Self> {
    fn dot(&self, v2: &T) -> f32;
    fn abs_dot(&self, b: &T) -> f32 { self.dot(b).abs() }
}

pub trait Normalize {
    fn normalize(self) -> Self;
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

////////////////////////////////////////////////////////////////////////////////
    
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Point {
    pub fn new() -> Point { Point { x: 0f32, y: 0f32, z: 0f32 } }
    pub fn new_with(_x: f32, _y: f32, _z: f32) -> Point {
        Point { x: _x, y: _y, z: _z }
    }
}

impl<'a, 'b> ::std::ops::Sub<&'b Vector> for &'a Point {
    type Output = Point;
    fn sub(self, other: &'b Vector) -> Point {
        Point::new_with(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<'a> ::std::ops::Sub<Vector> for &'a Point {
    type Output = Point;
    fn sub(self, other: Vector) -> Point { self - &other }
}

impl ::std::ops::Sub<Vector> for Point {
    type Output = Point;
    fn sub(self, _rhs: Vector) -> Point { &self - &_rhs }
}

impl<'a, 'b> ::std::ops::Sub<&'b Point> for &'a Point {
    type Output = Vector;
    fn sub(self, other: &'b Point) -> Vector {
        Vector::new_with(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<'a> ::std::ops::Sub<&'a Point> for Point {
    type Output = Vector;
    fn sub(self, other: &'a Point) -> Vector { &self - other }
}

impl<'a> ::std::ops::Sub<Point> for &'a Point {
    type Output = Vector;
    fn sub(self, other: Point) -> Vector { self - &other }
}

impl ::std::ops::Sub for Point {
    type Output = Vector;
    fn sub(self, other: Point) -> Vector { &self - &other}
}

impl<'a, 'b> ::std::ops::Add<&'b Vector> for &'a Point {
    type Output = Point;

    fn add(self, other: &'b Vector) -> Point {
        Point::new_with(self.x + other.x, self.y + other.y, self.x + other.x)
    }
}

impl<'a> ::std::ops::Add<Vector> for &'a Point {
    type Output = Point;
    fn add(self, _rhs: Vector) -> Point {
        Point::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl ::std::ops::Add<Vector> for Point {
    type Output = Point;
    fn add(self, _rhs: Vector) -> Point {
        Point::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a, 'b> ::std::ops::Add<&'b Point> for &'a Point {
    type Output = Point;

    fn add(self, other: &'b Point) -> Point {
        Point::new_with(self.x + other.x, self.y + other.y, self.x + other.x)
    }
}

impl<'a> ::std::ops::Add<Point> for &'a Point {
    type Output = Point;
    fn add(self, _rhs: Point) -> Point {
        Point::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl ::std::ops::Add<Point> for Point {
    type Output = Point;
    fn add(self, _rhs: Point) -> Point {
        Point::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl ::std::ops::Mul<f32> for Point {
    type Output = Point;
    fn mul(self, f: f32) -> Point {
        Point::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl<'a> ::std::ops::Mul<f32> for &'a Point {
    type Output = Point;
    fn mul(self, f: f32) -> Point {
        Point::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl ::std::ops::Mul<Point> for f32 {
    type Output = Point;
    fn mul(self, v: Point) -> Point { v * self }
}

impl<'a> ::std::ops::Mul<&'a Point> for f32 {
    type Output = Point;
    fn mul(self, v: &'a Point) -> Point { v * self }
}

impl ::std::ops::Index<i32> for Point {
    type Output = f32;
    fn index<'a>(&'a self, index: i32) -> &'a f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Error - Point index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<i32> for Point {
    fn index_mut<'a>(&'a mut self, index: i32) -> &'a mut f32 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Error - Point index out of bounds!")
        }
    }
}

impl Lerp<f32> for Point {
    fn lerp(&self, p: &Point, t: f32) -> Point {
        Point::new_with(
            self.x.lerp(&p.x, t),
            self.y.lerp(&p.y, t),
            self.z.lerp(&p.z, t))
    }
}

pub fn distance(p1: &Point, p2: &Point) -> f32 { (p1 - p2).length() }
pub fn distance_squared(p1: &Point, p2: &Point) -> f32 { (p1 - p2).length_squared() }

////////////////////////////////////////////////////////////////////////////////

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
