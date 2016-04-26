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

    pub fn face_forward(self, v: Vector) -> Normal {
        if self.clone().dot(&v) < 0f32 { -self } else { self }
    }

    pub fn cross(self, v2: &Normal) -> Normal {
        Normal::new_with(
            (self.y * v2.z) - (self.z * v2.y),
            (self.z * v2.x) - (self.x * v2.z),
            (self.x * v2.y) - (self.y * v2.x))
    }
}

impl<'a> ::std::convert::From<&'a Vector> for Normal {
    fn from(v: &'a Vector) -> Normal {
        Normal::new_with(v.x, v.y, v.z)        
    }
}

impl ::std::convert::From<Vector> for Normal {
    fn from(v: Vector) -> Normal { Normal::from(&v) }
}

impl<'a> ::std::convert::From<&'a Normal> for Vector {
    fn from(n: &'a Normal) -> Vector {
        Vector::new_with(n.x, n.y, n.z)        
    }
}

impl ::std::convert::From<Normal> for Vector {
    fn from(n: Normal) -> Vector { Vector::from(&n) }
}

impl<'a, 'b> ::std::ops::Sub<&'b Normal> for &'a Normal {
    type Output = Normal;
    fn sub(self, _rhs: &'b Normal) -> Normal {
        Normal::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl<'a> ::std::ops::Sub<&'a Normal> for Normal {
    type Output = Normal;
    fn sub(self, _rhs: &'a Normal) -> Normal { &self - _rhs }
}

impl<'a> ::std::ops::Sub<Normal> for &'a Normal {
    type Output = Normal;
    fn sub(self, _rhs:Normal) -> Normal { self - &_rhs }
}

impl ::std::ops::Sub for Normal {
    type Output = Normal;
    fn sub(self, _rhs: Normal) -> Normal {
        Normal::new_with(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl<'a, 'b> ::std::ops::Add<&'b Normal> for &'a Normal {
    type Output = Normal;
    fn add(self, _rhs: &'b Normal) -> Normal {
        Normal::new_with(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a> ::std::ops::Add<&'a Normal> for Normal {
    type Output = Normal;
    fn add(self, _rhs: &'a Normal) -> Normal { &self + _rhs }
}

impl<'a> ::std::ops::Add<Normal> for &'a Normal {
    type Output = Normal;
    fn add(self, _rhs:Normal) -> Normal { self + &_rhs }
}

impl ::std::ops::Add for Normal {
    type Output = Normal;
    fn add(self, _rhs: Normal) -> Normal { &self + &_rhs }
}

impl<'a> ::std::ops::Mul<f32> for &'a Normal {
    type Output = Vector;
    fn mul(self, f: f32) -> Vector {
        Vector::new_with(self.x * f, self.y * f, self.z * f)
    }
}

impl ::std::ops::Mul<f32> for Normal {
    type Output = Vector;
    fn mul(self, f: f32) -> Vector { &self * f }
}

impl ::std::ops::Mul<Normal> for f32 {
    type Output = Vector;
    fn mul(self, v: Normal) -> Vector { v * self }
}

impl<'a> ::std::ops::Mul<&'a Normal> for f32 {
    type Output = Vector;
    fn mul(self, v: &'a Normal) -> Vector { v * self }
}

impl<'a> ::std::ops::Div<f32> for &'a Normal {
    type Output = Vector;
    fn div(self, f: f32) -> Vector {
        let recip = 1f32 / f;
        recip * self
    }
}

impl ::std::ops::Div<f32> for Normal {
    type Output = Vector;
    fn div(self, f: f32) -> Vector { &self / f }
}

impl<'a> ::std::ops::Neg for &'a Normal {
    type Output = Normal;
    fn neg(self) -> Normal {
        Normal::new_with(-self.x, -self.y, -self.z)
    }
}

impl ::std::ops::Neg for Normal {
    type Output = Normal;
    fn neg(self) -> Normal { -&self }
}

impl ::std::ops::Index<usize> for Normal {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Error - Normal index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<usize> for Normal {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
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

impl Normalize for Vector {
    fn normalize(self) -> Vector {
        let l = self.length();
        self / l
    }
}

impl Normalize for Normal {
    fn normalize(self) -> Normal {
        let v = Vector::from(self);
        Normal::from(v.normalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32;
    use geometry::vector::Vector;
    use geometry::vector::Dot;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Normal::new(),
                   Normal { x: 0f32, y: 0f32, z: 0f32 });
    }

    #[test]
    fn it_can_be_created_from_values() {
        assert_eq!(Normal::new_with(0f32, 0f32, 0f32), Normal::new());
        assert_eq!(
            Normal::new_with(1f32, 2f32, 3f32),
            Normal { x: 1f32, y: 2f32, z: 3f32 });
        assert_eq!(
            Normal::new_with(-1f32, 2f32, -3f32),
            Normal { x: -1f32, y: 2f32, z: -3f32 });
    }

    #[test]
    fn it_can_be_subtracted() {
        let u = Normal::new_with(1f32, 2f32, 3f32);
        let v = Normal::new_with(4f32, 3f32, 2f32);

        assert_eq!(&u - &u, Normal::new());
        assert_eq!(&v - &v, Normal::new());

        assert_eq!(Normal::new() - &u, -(&u));
        assert_eq!(Normal::new() - &v, -(&v));

        assert_eq!(&u - &v, Normal::new_with(-3f32, -1f32, 1f32));
        assert_eq!(u.clone() - &v, Normal::new_with(-3f32, -1f32, 1f32));
        assert_eq!(&u - v.clone(), Normal::new_with(-3f32, -1f32, 1f32));
        assert_eq!(u.clone() - v.clone(), Normal::new_with(-3f32, -1f32, 1f32));

        assert_eq!(&v - &u, Normal::new_with(3f32, 1f32, -1f32));
        assert_eq!(v.clone() - &u, Normal::new_with(3f32, 1f32, -1f32));
        assert_eq!(&v - u.clone(), Normal::new_with(3f32, 1f32, -1f32));
        assert_eq!(v.clone() - u.clone(), Normal::new_with(3f32, 1f32, -1f32));
    }

    #[test]
    fn it_can_be_converted_to_and_from_vectors() {
        let u = Normal::new_with(1f32, 2f32, 3f32);
        let v = Normal::new_with(4f32, 3f32, 2f32);

        assert_eq!(Normal::from(Vector::from(u.clone())), u);
        assert_eq!(Normal::from(Vector::from(v.clone())), v);
    }

    
    #[test]
    fn it_can_be_added() {
        let u = Normal::new_with(1f32, 2f32, 3f32);
        let v = Normal::new_with(4f32, 3f32, 2f32);

        assert_eq!(Normal::new() + &u, u);
        assert_eq!(Normal::new() + &v, v);

        assert_eq!(&u + &v, Normal::new_with(5f32, 5f32, 5f32));
        assert_eq!(u.clone() + &v, Normal::new_with(5f32, 5f32, 5f32));
        assert_eq!(&u + v.clone(), Normal::new_with(5f32, 5f32, 5f32));
        assert_eq!(u.clone() + v.clone(), Normal::new_with(5f32, 5f32, 5f32));

        assert_eq!(&v + &u, Normal::new_with(5f32, 5f32, 5f32));
        assert_eq!(v.clone() + &u, Normal::new_with(5f32, 5f32, 5f32));
        assert_eq!(&v + u.clone(), Normal::new_with(5f32, 5f32, 5f32));
        assert_eq!(v.clone() + u.clone(), Normal::new_with(5f32, 5f32, 5f32));
    }

    #[test]
    fn it_can_be_scale() {
        for i in 0..100 {
            assert_eq!(Normal::new() * (i as f32), Vector::new());
        }

        let u = Normal::new_with(1f32, 2f32, 3f32);
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
            assert_eq!(Normal::new() / (i as f32), Vector::new());
        }

        let u = Normal::new_with(2f32, 4f32, 6f32);
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
        assert_eq!(-Normal::new(), Normal::new());
        assert_eq!(-(&Normal::new()), Normal::new());

        let u = Normal::new_with(2f32, 4f32, 6f32);
        let neg_u = Normal::new_with(-2f32, -4f32, -6f32);
        assert_eq!(-(&u), neg_u);
        assert_eq!(-u, neg_u);

        let infvec = Normal::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let nanvec = Normal::new_with(f32::NAN, f32::NAN, f32::NAN);

        assert_eq!(-(&infvec).x, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).y, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).z, f32::NEG_INFINITY);

        assert!((-(&nanvec)).x.is_nan());
        assert!((-(&nanvec)).y.is_nan());
        assert!((-(&nanvec)).z.is_nan());
    }

    #[test]
    fn it_can_be_indexed() {
        let mut v = Normal::new_with(-1f32, -1f32, 0f32);
        let iv = Normal::new_with(0.0001f32, 3f32, f32::consts::PI);

        v[0] = iv[0];
        v[1] = iv[1];
        v[2] = iv[2];
        assert_eq!(v, iv);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much() {
        let v = Normal::new_with(-1f32, -1f32, -1f32);
        println!("This should never appear: {:?}", v[3]);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either() {
        let mut v = Normal::new_with(-1f32, -1f32, -1f32);
        v[0] = 0f32;
        println!("This should never appear: {:?}", v[14]);
    }

    #[test]
    fn it_has_a_dot_product() {
        // Check perpendicular
        let x = Normal::new_with(1f32, 0f32, 0f32);
        let y = Normal::new_with(0f32, 1f32, 0f32);
        let z = Normal::new_with(0f32, 0f32, 1f32);
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

        let x = Normal::new_with(
            3f32 / (113f32.sqrt()),
            10f32 / (113f32.sqrt()),
            2f32 / (113f32.sqrt()));

        // If we subtract the perpendicular part of a vector... the remainder
        // should be perpendicular...
        assert!((&v - (x.dot(&v) * &x)).dot(&x).abs() < 1e-6f32);

        assert!(x.dot(&Normal::new_with(f32::NAN, 0f32, 0f32)).is_nan());
        assert!(x.dot(&Normal::new_with(0f32, f32::NAN, 0f32)).is_nan());
        assert!(x.dot(&Normal::new_with(0f32, 0f32, f32::NAN)).is_nan());

        assert_eq!(x.dot(&Normal::new_with(f32::INFINITY, 0f32, 0f32)), f32::INFINITY);
        assert_eq!(x.dot(&Normal::new_with(0f32, f32::INFINITY, 0f32)), f32::INFINITY);
        assert_eq!(x.dot(&Normal::new_with(0f32, 0f32, f32::INFINITY)), f32::INFINITY);
    }

    #[test]
    fn it_can_be_turned_around() {
        let n = Normal::new_with(1f32, 0f32, 0f32);

        assert_eq!(n.clone().face_forward(Vector::new_with(-1f32, -1f32, -1f32)), -(&n));
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, 1f32, 1f32)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(f32::NAN, 1f32, 1f32)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, f32::NAN, 1f32)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, 1f32, f32::NAN)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(f32::INFINITY, 1f32, 1f32)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, f32::INFINITY, 1f32)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, 1f32, f32::INFINITY)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(f32::NEG_INFINITY, 1f32, 1f32)), -(&n));
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, f32::NEG_INFINITY, 1f32)), n);
        assert_eq!(n.clone().face_forward(Vector::new_with(1f32, 1f32, f32::NEG_INFINITY)), n);
    }
}
