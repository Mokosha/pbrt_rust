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

    pub fn distance(&self, p: &Point) -> f32 { (self - p).length() }
    pub fn distance_squared(&self, p: &Point) -> f32 { (self - p).length_squared() }
}

impl From<Point> for Vector {
    fn from(p: Point) -> Vector { Vector::new_with(p.x, p.y, p.z) }
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

impl<'a> ::std::ops::Sub<&'a Vector> for Point {
    type Output = Point;
    fn sub(self, other: &'a Vector) -> Point { &self - other }
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
        Point::new_with(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<'a> ::std::ops::Add<Vector> for &'a Point {
    type Output = Point;
    fn add(self, _rhs: Vector) -> Point { self + &_rhs }
}

impl<'a> ::std::ops::Add<&'a Vector> for Point {
    type Output = Point;
    fn add(self, _rhs: &'a Vector) -> Point { &self + _rhs }
}

impl ::std::ops::Add<Vector> for Point {
    type Output = Point;
    fn add(self, _rhs: Vector) -> Point { &self + &_rhs }
}

impl<'a, 'b> ::std::ops::Add<&'b Point> for &'a Point {
    type Output = Point;

    fn add(self, other: &'b Point) -> Point {
        Point::new_with(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<'a> ::std::ops::Add<Point> for &'a Point {
    type Output = Point;
    fn add(self, _rhs: Point) -> Point { self + &_rhs }
}

impl<'a> ::std::ops::Add<&'a Point> for Point {
    type Output = Point;
    fn add(self, _rhs: &'a Point) -> Point { &self + _rhs }
}

impl ::std::ops::Add<Point> for Point {
    type Output = Point;
    fn add(self, _rhs: Point) -> Point { &self + &_rhs }
}

impl<'a> ::std::ops::Mul<f32> for &'a Point {
    type Output = Point;
    fn mul(self, f: f32) -> Point {
        Point::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl ::std::ops::Mul<f32> for Point {
    type Output = Point;
    fn mul(self, f: f32) -> Point { &self * f }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32;
    use utils::Lerp;
    use geometry::vector::Vector;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Point::new(),
                   Point { x: 0f32, y: 0f32, z: 0f32 });
    }

    #[test]
    fn it_can_be_created_from_values() {
        assert_eq!(Point::new_with(0f32, 0f32, 0f32), Point::new());
        assert_eq!(
            Point::new_with(1f32, 2f32, 3f32),
            Point { x: 1f32, y: 2f32, z: 3f32 });
        assert_eq!(
            Point::new_with(-1f32, 2f32, -3f32),
            Point { x: -1f32, y: 2f32, z: -3f32 });
    }

    #[test]
    fn it_can_be_converted_to_a_vector() {
        assert_eq!(Vector::new(), Vector::from(Point::new()));
        assert_eq!(Vector::new_with(1.0, 2.0, 3.0),
                   Vector::from(Point::new_with(1.0, 2.0, 3.0)));
        assert_eq!(Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY),
                   Vector::from(Point::new_with(f32::INFINITY,
                                                f32::INFINITY,
                                                f32::INFINITY)));
    }

    #[test]
    fn they_have_squared_distances() {
        assert_eq!(Point::new().distance_squared(&Point::new()), 0f32);
        assert!(Point::new_with(f32::NAN, 0f32, 0f32).distance_squared(&Point::new()).is_nan());
        assert!(Point::new_with(0f32, f32::NAN, 0f32).distance_squared(&Point::new()).is_nan());
        assert!(Point::new_with(0f32, 0f32, f32::NAN).distance_squared(&Point::new()).is_nan());
        assert_eq!(Point::new_with(1f32, 0f32, 0f32).distance_squared(&Point::new()), 1f32);
        assert_eq!(Point::new_with(-1f32, 0f32, 0f32).distance_squared(&Point::new()), 1f32);
        assert_eq!(Point::new_with(1f32, 1f32, 1f32).distance_squared(&Point::new()), 3f32);
        assert_eq!(Point::new_with(0f32, 2f32, 0f32).distance_squared(&Point::new()), 4f32);
        assert_eq!(Point::new_with(0f32, 0f32, 2f32).distance_squared(&Point::new()), 4f32);
        assert_eq!(Point::new_with(f32::INFINITY, 0f32, 0f32).distance_squared(&Point::new()), f32::INFINITY);
        assert_eq!(Point::new_with(0f32, f32::INFINITY, 0f32).distance_squared(&Point::new()), f32::INFINITY);
        assert_eq!(Point::new_with(0f32, 0f32, f32::INFINITY).distance_squared(&Point::new()), f32::INFINITY);
    }

    #[test]
    fn they_have_distance() {
        assert_eq!(Point::new().distance(&Point::new()), 0f32);
        assert!(Point::new_with(f32::NAN, 0f32, 0f32).distance(&Point::new()).is_nan());
        assert!(Point::new_with(0f32, f32::NAN, 0f32).distance(&Point::new()).is_nan());
        assert!(Point::new_with(0f32, 0f32, f32::NAN).distance(&Point::new()).is_nan());
        assert_eq!(Point::new_with(1f32, 0f32, 0f32).distance(&Point::new()), 1f32);
        assert_eq!(Point::new_with(-1f32, 0f32, 0f32).distance(&Point::new()), 1f32);
        assert_eq!(Point::new_with(1f32, 1f32, 1f32).distance(&Point::new()), 3f32.sqrt());
        assert_eq!(Point::new_with(0f32, 2f32, 0f32).distance(&Point::new()), 2f32);
        assert_eq!(Point::new_with(0f32, 0f32, 2f32).distance(&Point::new()), 2f32);
        assert_eq!(Point::new_with(f32::INFINITY, 0f32, 0f32).distance(&Point::new()), f32::INFINITY);
        assert_eq!(Point::new_with(0f32, f32::INFINITY, 0f32).distance(&Point::new()), f32::INFINITY);
        assert_eq!(Point::new_with(0f32, 0f32, f32::INFINITY).distance(&Point::new()), f32::INFINITY);
    }

    #[test]
    fn it_can_be_subtracted() {
        let u = Point::new_with(1f32, 2f32, 3f32);
        let v = Point::new_with(4f32, 3f32, 2f32);

        assert_eq!(&u - &u, Vector::new());
        assert_eq!(&v - &v, Vector::new());

        assert_eq!(Point::new() - &u, Vector::new_with(-1f32, -2f32, -3f32));
        assert_eq!(Point::new() - &v, Vector::new_with(-4f32, -3f32, -2f32));

        assert_eq!(&u - &v, Vector::new_with(-3f32, -1f32, 1f32));
        assert_eq!(u.clone() - &v, Vector::new_with(-3f32, -1f32, 1f32));
        assert_eq!(&u - v.clone(), Vector::new_with(-3f32, -1f32, 1f32));
        assert_eq!(u.clone() - v.clone(), Vector::new_with(-3f32, -1f32, 1f32));

        assert_eq!(&v - &u, Vector::new_with(3f32, 1f32, -1f32));
        assert_eq!(v.clone() - &u, Vector::new_with(3f32, 1f32, -1f32));
        assert_eq!(&v - u.clone(), Vector::new_with(3f32, 1f32, -1f32));
        assert_eq!(v.clone() - u.clone(), Vector::new_with(3f32, 1f32, -1f32));
    }

    #[test]
    fn it_can_be_added() {
        let u = Point::new_with(1f32, 2f32, 3f32);
        let v = Point::new_with(4f32, 3f32, 2f32);

        assert_eq!(Point::new() + &u, u);
        assert_eq!(Point::new() + &v, v);

        assert_eq!(&u + &v, Point::new_with(5f32, 5f32, 5f32));
        assert_eq!(u.clone() + &v, Point::new_with(5f32, 5f32, 5f32));
        assert_eq!(&u + v.clone(), Point::new_with(5f32, 5f32, 5f32));
        assert_eq!(u.clone() + v.clone(), Point::new_with(5f32, 5f32, 5f32));

        assert_eq!(&v + &u, Point::new_with(5f32, 5f32, 5f32));
        assert_eq!(v.clone() + &u, Point::new_with(5f32, 5f32, 5f32));
        assert_eq!(&v + u.clone(), Point::new_with(5f32, 5f32, 5f32));
        assert_eq!(v.clone() + u.clone(), Point::new_with(5f32, 5f32, 5f32));
    }

    #[test]
    fn it_can_be_scaled() {
        for i in (0..100) {
            assert_eq!(Point::new() * (i as f32), Point::new());
        }

        let u = Point::new_with(1f32, 2f32, 3f32);
        let f = 2f32;
        let scaled_u = Point::new_with(2f32, 4f32, 6f32);
        let scaled_neg_u = Point::new_with(-2f32, -4f32, -6f32);

        assert_eq!(&u * f, scaled_u);
        assert_eq!(u.clone() * f, scaled_u);
        assert_eq!(f * &u, scaled_u);
        assert_eq!(f * u.clone(), scaled_u);

        assert_eq!(&u * -f, scaled_neg_u);
        assert_eq!(u.clone() * -f, scaled_neg_u);
        assert_eq!(-f * &u, scaled_neg_u);
        assert_eq!(-f * u.clone(), scaled_neg_u);

        assert!((f32::NAN * u.clone()).x.is_nan());
        assert!((f32::NAN * u.clone()).y.is_nan());
        assert!((f32::NAN * u.clone()).z.is_nan());
    }

    #[test]
    fn it_can_be_indexed() {
        let mut v = Point::new_with(-1f32, -1f32, 0f32);
        let iv = Point::new_with(0.0001f32, 3f32, f32::consts::PI);

        v[0] = iv[0];
        v[1] = iv[1];
        v[2] = iv[2];
        assert_eq!(v, iv);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much() {
        let v = Point::new_with(-1f32, -1f32, -1f32);
        println!("This should never appear: {:?}", v[3]);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either() {
        let mut v = Point::new_with(-1f32, -1f32, -1f32);
        v[0] = 0f32;
        println!("This should never appear: {:?}", v[14]);
    }

    #[test]
    fn it_can_be_interpolated() {
        let x = Point::new_with(1f32, 0f32, 0f32);
        let y = Point::new_with(0f32, 1f32, 0f32);

        assert_eq!(x.lerp(&y, 0f32), x);
        assert!((x.lerp(&y, 0.1f32) - Point::new_with(0.9f32, 0.1f32, 0f32)).length_squared() < 1e-6f32);
        assert_eq!(x.lerp(&y, 0.5f32), y.lerp(&x, 0.5f32));
        assert!((x.lerp(&y, 0.9f32) - Point::new_with(0.1f32, 0.9f32, 0f32)).length_squared() < 1e-6f32);
        assert_eq!(x.lerp(&y, 1f32), y);
    }
}
