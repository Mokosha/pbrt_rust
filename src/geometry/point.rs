use geometry::vector::Vector;
use utils::Lerp;

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
