use utils::Lerp;

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

impl<'a, 'b> ::std::ops::Sub<&'b Vector> for &'a Vector {
    type Output = Vector;
    fn sub(self, _rhs: &'b Vector) -> Vector {
        Vector::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl<'a> ::std::ops::Sub<&'a Vector> for Vector {
    type Output = Vector;
    fn sub(self, _rhs: &'a Vector) -> Vector { &self - _rhs }
}

impl<'a> ::std::ops::Sub<Vector> for &'a Vector {
    type Output = Vector;
    fn sub(self, _rhs: Vector) -> Vector { self - &_rhs }
}

impl ::std::ops::Sub for Vector {
    type Output = Vector;
    fn sub(self, _rhs: Vector) -> Vector { &self - &_rhs }
}

impl<'a, 'b> ::std::ops::Add<&'b Vector> for &'a Vector {
    type Output = Vector;
    fn add(self, _rhs: &'b Vector) -> Vector {
        Vector::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a> ::std::ops::Add<Vector> for &'a Vector {
    type Output = Vector;
    fn add(self, _rhs: Vector) -> Vector { self + &_rhs }
}

impl<'a> ::std::ops::Add<&'a Vector> for Vector {
    type Output = Vector;
    fn add(self, _rhs: &'a Vector) -> Vector { &self + _rhs }
}

impl ::std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, _rhs: Vector) -> Vector { &self + &_rhs }
}

impl<'a> ::std::ops::Mul<f32> for &'a Vector {
    type Output = Vector;
    fn mul(self, f: f32) -> Vector {
        Vector::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl ::std::ops::Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, f: f32) -> Vector { &self * f }
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

impl ::std::ops::Index<usize> for Vector {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Error - Vector index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<usize> for Vector {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
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

impl Lerp<f32> for Vector {
    fn lerp(&self, b: &Vector, t: f32) -> Vector {
        (1f32 - t) * self + t * b
    }
}

pub fn coordinate_system(v1: &Vector) -> (Vector, Vector) {
    let v2 =
        if v1.x.abs() > v1.y.abs() {
            let inv_len = 1f32 / ((v1.x * v1.x + v1.z * v1.z).sqrt());
            Vector::new_with(-v1.x * inv_len, 0f32, v1.x * inv_len)
        } else {
            let inv_len = 1f32 / ((v1.y * v1.y + v1.z * v1.z).sqrt());
            Vector::new_with(0f32, v1.z * inv_len, -v1.y * inv_len)
        };
    let v3 = v1.clone().cross(&v2);
    (v3.clone().cross(&v1), v3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32;
    use utils::Lerp;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Vector::new(),
                   Vector { x: 0f32, y: 0f32, z: 0f32 });
    }

    #[test]
    fn it_can_be_created_from_values() {
        assert_eq!(Vector::new_with(0f32, 0f32, 0f32), Vector::new());
        assert_eq!(
            Vector::new_with(1f32, 2f32, 3f32),
            Vector { x: 1f32, y: 2f32, z: 3f32 });
        assert_eq!(
            Vector::new_with(-1f32, 2f32, -3f32),
            Vector { x: -1f32, y: 2f32, z: -3f32 });
    }

    #[test]
    fn it_has_a_squared_length() {
        assert_eq!(Vector::new().length_squared(), 0f32);
        assert!(Vector::new_with(f32::NAN, 0f32, 0f32).length_squared().is_nan());
        assert!(Vector::new_with(0f32, f32::NAN, 0f32).length_squared().is_nan());
        assert!(Vector::new_with(0f32, 0f32, f32::NAN).length_squared().is_nan());
        assert_eq!(Vector::new_with(1f32, 0f32, 0f32).length_squared(), 1f32);
        assert_eq!(Vector::new_with(-1f32, 0f32, 0f32).length_squared(), 1f32);
        assert_eq!(Vector::new_with(1f32, 1f32, 1f32).length_squared(), 3f32);
        assert_eq!(Vector::new_with(0f32, 2f32, 0f32).length_squared(), 4f32);
        assert_eq!(Vector::new_with(0f32, 0f32, 2f32).length_squared(), 4f32);
        assert_eq!(Vector::new_with(f32::INFINITY, 0f32, 0f32).length_squared(), f32::INFINITY);
        assert_eq!(Vector::new_with(0f32, f32::INFINITY, 0f32).length_squared(), f32::INFINITY);
        assert_eq!(Vector::new_with(0f32, 0f32, f32::INFINITY).length_squared(), f32::INFINITY);
    }

    #[test]
    fn it_has_length() {
        assert_eq!(Vector::new().length(), 0f32);
        assert!(Vector::new_with(f32::NAN, 0f32, 0f32).length().is_nan());
        assert!(Vector::new_with(0f32, f32::NAN, 0f32).length().is_nan());
        assert!(Vector::new_with(0f32, 0f32, f32::NAN).length().is_nan());
        assert_eq!(Vector::new_with(1f32, 0f32, 0f32).length(), 1f32);
        assert_eq!(Vector::new_with(-1f32, 0f32, 0f32).length(), 1f32);
        assert_eq!(Vector::new_with(1f32, 1f32, 1f32).length(), 3f32.sqrt());
        assert_eq!(Vector::new_with(0f32, 2f32, 0f32).length(), 2f32);
        assert_eq!(Vector::new_with(0f32, 0f32, 2f32).length(), 2f32);
        assert_eq!(Vector::new_with(f32::INFINITY, 0f32, 0f32).length(), f32::INFINITY);
        assert_eq!(Vector::new_with(0f32, f32::INFINITY, 0f32).length(), f32::INFINITY);
        assert_eq!(Vector::new_with(0f32, 0f32, f32::INFINITY).length(), f32::INFINITY);
    }

    #[test]
    fn it_can_take_a_cross_product() {
        let x = Vector::new_with(1f32, 0f32, 0f32);
        let y = Vector::new_with(0f32, 1f32, 0f32);
        let z = Vector::new_with(0f32, 0f32, 1f32);

        assert_eq!(x.clone().cross(&y), z);
        assert_eq!(y.clone().cross(&x), -(&z));

        assert_eq!(y.clone().cross(&z), x);
        assert_eq!(z.clone().cross(&y), -(&x));

        assert_eq!(z.clone().cross(&x), y);
        assert_eq!(x.clone().cross(&z), -(&y));

        assert_eq!(Vector::new_with(1f32, 1f32, 0f32).cross(&x), -(&z));
        assert_eq!(x.clone().cross(&Vector::new_with(1f32, 1f32, 0f32)), z);
    }

    #[test]
    fn it_can_be_subtracted() {
        let u = Vector::new_with(1f32, 2f32, 3f32);
        let v = Vector::new_with(4f32, 3f32, 2f32);

        assert_eq!(&u - &u, Vector::new());
        assert_eq!(&v - &v, Vector::new());

        assert_eq!(Vector::new() - &u, -(&u));
        assert_eq!(Vector::new() - &v, -(&v));

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
        let u = Vector::new_with(1f32, 2f32, 3f32);
        let v = Vector::new_with(4f32, 3f32, 2f32);

        assert_eq!(Vector::new() + &u, u);
        assert_eq!(Vector::new() + &v, v);

        assert_eq!(&u + &v, Vector::new_with(5f32, 5f32, 5f32));
        assert_eq!(u.clone() + &v, Vector::new_with(5f32, 5f32, 5f32));
        assert_eq!(&u + v.clone(), Vector::new_with(5f32, 5f32, 5f32));
        assert_eq!(u.clone() + v.clone(), Vector::new_with(5f32, 5f32, 5f32));

        assert_eq!(&v + &u, Vector::new_with(5f32, 5f32, 5f32));
        assert_eq!(v.clone() + &u, Vector::new_with(5f32, 5f32, 5f32));
        assert_eq!(&v + u.clone(), Vector::new_with(5f32, 5f32, 5f32));
        assert_eq!(v.clone() + u.clone(), Vector::new_with(5f32, 5f32, 5f32));
    }

    #[test]
    fn it_can_be_scale() {
        for i in 0..100 {
            assert_eq!(Vector::new() * (i as f32), Vector::new());
        }

        let u = Vector::new_with(1f32, 2f32, 3f32);
        let f = 2f32;
        let scaled_u = Vector::new_with(2f32, 4f32, 6f32);
        let scaled_neg_u = Vector::new_with(-2f32, -4f32, -6f32);

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
    fn it_can_be_divided_by_scalars() {
        for i in 1..100 {
            assert_eq!(Vector::new() / (i as f32), Vector::new());
        }

        let u = Vector::new_with(2f32, 4f32, 6f32);
        let f = 2f32;
        let scaled_u = Vector::new_with(1f32, 2f32, 3f32);
        let scaled_neg_u = Vector::new_with(-1f32, -2f32, -3f32);

        assert_eq!(&u / f, scaled_u);
        assert_eq!(u.clone() / f, scaled_u);

        assert_eq!(&u / -f, scaled_neg_u);
        assert_eq!(u.clone() / -f, scaled_neg_u);

        assert!((u.clone() / f32::NAN).x.is_nan());
        assert!((u.clone() / f32::NAN).y.is_nan());
        assert!((u.clone() / f32::NAN).z.is_nan());
    }

    #[test]
    fn it_can_be_negated() {
        assert_eq!(-Vector::new(), Vector::new());
        assert_eq!(-(&Vector::new()), Vector::new());

        let u = Vector::new_with(2f32, 4f32, 6f32);
        let neg_u = Vector::new_with(-2f32, -4f32, -6f32);
        assert_eq!(-(&u), neg_u);
        assert_eq!(-u, neg_u);

        let infvec = Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let nanvec = Vector::new_with(f32::NAN, f32::NAN, f32::NAN);

        assert_eq!(-(&infvec).x, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).y, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).z, f32::NEG_INFINITY);

        assert!((-(&nanvec)).x.is_nan());
        assert!((-(&nanvec)).y.is_nan());
        assert!((-(&nanvec)).z.is_nan());
    }

    #[test]
    fn it_can_be_indexed() {
        let mut v = Vector::new_with(-1f32, -1f32, 0f32);
        let iv = Vector::new_with(0.0001f32, 3f32, f32::consts::PI);

        v[0] = iv[0];
        v[1] = iv[1];
        v[2] = iv[2];
        assert_eq!(v, iv);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much() {
        let v = Vector::new_with(-1f32, -1f32, -1f32);
        println!("This should never appear: {:?}", v[3]);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either() {
        let mut v = Vector::new_with(-1f32, -1f32, -1f32);
        v[0] = 0f32;
        println!("This should never appear: {:?}", v[14]);
    }

    #[test]
    fn it_has_a_dot_product() {
        // Check perpendicular
        let x = Vector::new_with(1f32, 0f32, 0f32);
        let y = Vector::new_with(0f32, 1f32, 0f32);
        let z = Vector::new_with(0f32, 0f32, 1f32);
        assert_eq!(x.dot(&y), 0f32);
        assert_eq!(y.dot(&z), 0f32);
        assert_eq!(z.dot(&x), 0f32);

        // Check not perpendicular...
        assert_eq!(x.dot(&x), 1f32);
        assert_eq!(y.dot(&y), 1f32);
        assert_eq!(z.dot(&z), 1f32);

        // Check decomposition...
        let v = Vector::new_with(2f32, 3f32, 4f32);
        assert_eq!(x.dot(&v), 2f32);
        assert_eq!(y.dot(&v), 3f32);
        assert_eq!(z.dot(&v), 4f32);

        let x = Vector::new_with(
            3f32 / (113f32.sqrt()),
            10f32 / (113f32.sqrt()),
            2f32 / (113f32.sqrt()));

        // If we subtract the perpendicular part of a vector... the remainder
        // should be perpendicular...
        assert!((&v - (x.dot(&v) * &x)).dot(&x).abs() < 1e-6f32);

        assert!(x.dot(&Vector::new_with(f32::NAN, 0f32, 0f32)).is_nan());
        assert!(x.dot(&Vector::new_with(0f32, f32::NAN, 0f32)).is_nan());
        assert!(x.dot(&Vector::new_with(0f32, 0f32, f32::NAN)).is_nan());

        assert_eq!(x.dot(&Vector::new_with(f32::INFINITY, 0f32, 0f32)), f32::INFINITY);
        assert_eq!(x.dot(&Vector::new_with(0f32, f32::INFINITY, 0f32)), f32::INFINITY);
        assert_eq!(x.dot(&Vector::new_with(0f32, 0f32, f32::INFINITY)), f32::INFINITY);
    }

    #[test]
    fn it_can_be_interpolated() {
        let x = Vector::new_with(1f32, 0f32, 0f32);
        let y = Vector::new_with(0f32, 1f32, 0f32);

        assert_eq!(x.lerp(&y, 0f32), x);
        assert!((x.lerp(&y, 0.1f32) - Vector::new_with(0.9f32, 0.1f32, 0f32)).length_squared() < 1e-6f32);
        assert_eq!(x.lerp(&y, 0.5f32), y.lerp(&x, 0.5f32));
        assert!((x.lerp(&y, 0.9f32) - Vector::new_with(0.1f32, 0.9f32, 0f32)).length_squared() < 1e-6f32);
        assert_eq!(x.lerp(&y, 1f32), y);
    }

    #[test]
    fn it_can_generate_a_coordinate_system() {
        // All we care about here is the mutual perpendicularity of the three vectors.
        let v = Vector::new_with(3f32, -1f32, 0.0003f32);
        let (x, y) = coordinate_system(&v);

        assert!(v.dot(&x).abs() < 1e-6f32);
        assert!(v.dot(&y).abs() < 1e-6f32);
        assert!(x.dot(&y).abs() < 1e-6f32);
    }
}
