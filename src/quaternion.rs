use geometry::normal::Normalize;
use geometry::vector::Dot;
use geometry::vector::Vector;
use utils::Clamp;
use utils::Lerp;

#[derive(Debug, Clone, PartialEq)]
pub struct Quaternion {
    pub v: Vector,
    pub w: f32
}

impl Quaternion {
    pub fn new() -> Quaternion {
        Quaternion { v: Vector::new(), w: 1f32 }
    }

    pub fn new_with(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { v: Vector::new_with(x, y, z), w: w }
    }
}

impl<'a, 'b> ::std::ops::Add<&'b Quaternion> for &'a Quaternion {
    type Output = Quaternion;
    fn add(self, q: &'b Quaternion) -> Quaternion {
        Quaternion::new_with(
            self.v.x + q.v.x,
            self.v.y + q.v.y,
            self.v.z + q.v.z,
            self.w + q.w)
    }
}

impl<'a> ::std::ops::Add<Quaternion> for &'a Quaternion {
    type Output = Quaternion;
    fn add(self, q: Quaternion) -> Quaternion { self + &q }
}

impl<'a> ::std::ops::Add<&'a Quaternion> for Quaternion {
    type Output = Quaternion;
    fn add(self, q: &'a Quaternion) -> Quaternion { &self + q }
}

impl ::std::ops::Add for Quaternion {
    type Output = Quaternion;
    fn add(self, q: Quaternion) -> Quaternion { &self + &q }
}

impl<'a, 'b> ::std::ops::Sub<&'b Quaternion> for &'a Quaternion {
    type Output = Quaternion;
    fn sub(self, q: &'b Quaternion) -> Quaternion {
        Quaternion::new_with(
            self.v.x - q.v.x,
            self.v.y - q.v.y,
            self.v.z - q.v.z,
            self.w - q.w)
    }
}

impl<'a> ::std::ops::Sub<Quaternion> for &'a Quaternion {
    type Output = Quaternion;
    fn sub(self, q: Quaternion) -> Quaternion { self - &q }
}

impl<'a> ::std::ops::Sub<&'a Quaternion> for Quaternion {
    type Output = Quaternion;
    fn sub(self, q: &'a Quaternion) -> Quaternion { &self - q }
}

impl ::std::ops::Sub for Quaternion {
    type Output = Quaternion;
    fn sub(self, q: Quaternion) -> Quaternion { &self - &q }
}


impl<'a> ::std::ops::Mul<f32> for &'a Quaternion {
    type Output = Quaternion;
    fn mul(self, s: f32) -> Quaternion {
        Quaternion::new_with(
            self.v.x * s,
            self.v.y * s,
            self.v.z * s,
            self.w * s)
    }
}

impl<'a> ::std::ops::Mul<&'a Quaternion> for f32 {
    type Output = Quaternion;
    fn mul(self, q: &'a Quaternion) -> Quaternion { q * self }
}

impl ::std::ops::Mul<f32> for Quaternion {
    type Output = Quaternion;
    fn mul(self, s: f32) -> Quaternion { &self * s }
}

impl ::std::ops::Mul<Quaternion> for f32 {
    type Output = Quaternion;
    fn mul(self, q: Quaternion) -> Quaternion { &q * self }
}

impl Dot for Quaternion {
    fn dot(&self, q: &Quaternion) -> f32 {
        self.v.dot(&q.v) + self.w * q.w
    }
}

impl Normalize for Quaternion {
    fn normalize(self) -> Quaternion {
        let len = self.dot(&self).sqrt();
        Quaternion::new_with(
            self.v.x / len, self.v.y / len, self.v.z / len, self.w / len)
    }
}

impl Lerp<f32> for Quaternion {
    fn lerp(&self, q: &Quaternion, t: f32) -> Quaternion {
        let cos_theta = self.dot(q);
        if (cos_theta > 0.9995f32) {
            ((1f32 - t) * self + t * q).normalize()
        } else {
            let thetap = cos_theta.clamp(-1f32, 1f32).acos() * t;
            let qperp = (q - self * cos_theta).normalize();
            self * thetap.cos() + qperp * thetap.sin()
        }
    }
}

impl ::std::ops::Index<i32> for Quaternion {
    type Output = f32;
    fn index<'a>(&'a self, index: i32) -> &'a f32 {
        match index {
            0 => &self.v.x,
            1 => &self.v.y,
            2 => &self.v.z,
            3 => &self.w,
            _ => panic!("Error - Quaternion index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<i32> for Quaternion {
    fn index_mut<'a>(&'a mut self, index: i32) -> &'a mut f32 {
        match index {
            0 => &mut self.v.x,
            1 => &mut self.v.y,
            2 => &mut self.v.z,
            3 => &mut self.w,
            _ => panic!("Error - Quaternion index out of bounds!")
        }
    }
}


impl<'a> ::std::ops::Neg for &'a Quaternion {
    type Output = Quaternion;
    fn neg(self) -> Quaternion {
        Quaternion::new_with(-self.v.x, -self.v.y, -self.v.z, -self.w)
    }
}

impl ::std::ops::Neg for Quaternion {
    type Output = Quaternion;
    fn neg(self) -> Quaternion { -(&self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::vector::Vector;
    use std::f32;
    use utils::Lerp;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Quaternion::new(),
                   Quaternion { v: Vector::new(), w: 1f32 });
    }

    #[test]
    fn it_can_be_created_from_values() {
        assert_eq!(Quaternion::new_with(0f32, 0f32, 0f32, 1f32), Quaternion::new());
        assert_eq!(
            Quaternion::new_with(1f32, 2f32, 3f32, 4f32),
            Quaternion { v: Vector { x: 1f32, y: 2f32, z: 3f32 }, w: 4f32 });
        assert_eq!(
            Quaternion::new_with(-1f32, 2f32, -3f32, -4f32),
            Quaternion { v: Vector { x: -1f32, y: 2f32, z: -3f32 }, w: -4f32 });
    }

    #[test]
    fn it_can_be_subtracted() {
        let u = Quaternion::new_with(1f32, 2f32, 3f32, 4f32);
        let v = Quaternion::new_with(4f32, 3f32, 2f32, 1f32);

        let zero = Quaternion::new_with(0f32, 0f32, 0f32, 0f32);
        assert_eq!(&u - &u, zero);
        assert_eq!(&v - &v, zero);

        assert_eq!(&zero - &u, -(&u));
        assert_eq!(&zero - &v, -(&v));

        assert_eq!(&u - &v, Quaternion::new_with(-3f32, -1f32, 1f32, 3f32));
        assert_eq!(u.clone() - &v, Quaternion::new_with(-3f32, -1f32, 1f32, 3f32));
        assert_eq!(&u - v.clone(), Quaternion::new_with(-3f32, -1f32, 1f32, 3f32));
        assert_eq!(u.clone() - v.clone(), Quaternion::new_with(-3f32, -1f32, 1f32, 3f32));

        assert_eq!(&v - &u, Quaternion::new_with(3f32, 1f32, -1f32, -3f32));
        assert_eq!(v.clone() - &u, Quaternion::new_with(3f32, 1f32, -1f32, -3f32));
        assert_eq!(&v - u.clone(), Quaternion::new_with(3f32, 1f32, -1f32, -3f32));
        assert_eq!(v.clone() - u.clone(), Quaternion::new_with(3f32, 1f32, -1f32, -3f32));
    }

    #[test]
    fn it_can_be_added() {
        let u = Quaternion::new_with(1f32, 2f32, 3f32, 4f32);
        let v = Quaternion::new_with(4f32, 3f32, 2f32, 1f32);

        let zero = Quaternion::new_with(0f32, 0f32, 0f32, 0f32);
        assert_eq!(&zero + &u, u);
        assert_eq!(&zero + &v, v);

        assert_eq!(&u + &v, Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
        assert_eq!(u.clone() + &v, Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
        assert_eq!(&u + v.clone(), Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
        assert_eq!(u.clone() + v.clone(), Quaternion::new_with(5f32, 5f32, 5f32, 5f32));

        assert_eq!(&v + &u, Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
        assert_eq!(v.clone() + &u, Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
        assert_eq!(&v + u.clone(), Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
        assert_eq!(v.clone() + u.clone(), Quaternion::new_with(5f32, 5f32, 5f32, 5f32));
    }

    #[test]
    fn it_can_be_scaled() {
        for i in (0..100) {
            assert_eq!(Quaternion::new() * (i as f32), Quaternion::new_with(0f32, 0f32, 0f32, i as f32));
        }

        let u = Quaternion::new_with(1f32, 2f32, 3f32, 4f32);
        let f = 2f32;
        let scaled_u = Quaternion::new_with(2f32, 4f32, 6f32, 8f32);
        let scaled_neg_u = Quaternion::new_with(-2f32, -4f32, -6f32, -8f32);

        assert_eq!(&u * f, scaled_u);
        assert_eq!(u.clone() * f, scaled_u);
        assert_eq!(f * &u, scaled_u);
        assert_eq!(f * u.clone(), scaled_u);

        assert_eq!(&u * -f, scaled_neg_u);
        assert_eq!(u.clone() * -f, scaled_neg_u);
        assert_eq!(-f * &u, scaled_neg_u);
        assert_eq!(-f * u.clone(), scaled_neg_u);

        assert!((f32::NAN * u.clone()).v.x.is_nan());
        assert!((f32::NAN * u.clone()).v.y.is_nan());
        assert!((f32::NAN * u.clone()).v.z.is_nan());
        assert!((f32::NAN * u.clone()).w.is_nan());
    }

    #[test]
    fn it_can_be_negated() {
        assert_eq!(-Quaternion::new(), Quaternion::new_with(0f32, 0f32, 0f32, -1f32));
        assert_eq!(-(&Quaternion::new()), Quaternion::new_with(0f32, 0f32, 0f32, -1f32));

        let u = Quaternion::new_with(2f32, 4f32, 6f32, 8f32);
        let neg_u = Quaternion::new_with(-2f32, -4f32, -6f32, -8f32);
        assert_eq!(-(&u), neg_u);
        assert_eq!(-u, neg_u);

        let infvec = Quaternion::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let nanvec = Quaternion::new_with(f32::NAN, f32::NAN, f32::NAN, f32::NAN);

        assert_eq!(-(&infvec).v.x, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).v.y, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).v.z, f32::NEG_INFINITY);
        assert_eq!(-(&infvec).w, f32::NEG_INFINITY);

        assert!((-(&nanvec)).v.x.is_nan());
        assert!((-(&nanvec)).v.y.is_nan());
        assert!((-(&nanvec)).v.z.is_nan());
        assert!((-(&nanvec)).w.is_nan());
    }

    #[test]
    fn it_can_be_indexed() {
        let mut v = Quaternion::new_with(-1f32, -1f32, 0f32, 0.2f32);
        let iv = Quaternion::new_with(0.0001f32, 3f32, f32::consts::PI, -14f32);

        v[0] = iv[0];
        v[1] = iv[1];
        v[2] = iv[2];
        v[3] = iv[3];
        assert_eq!(v, iv);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much() {
        let v = Quaternion::new_with(-1f32, -1f32, -1f32, -1f32);
        println!("This should never appear: {:?}", v[4]);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either() {
        let mut v = Quaternion::new_with(-1f32, -1f32, -1f32, -1f32);
        v[0] = 0f32;
        println!("This should never appear: {:?}", v[14]);
    }

    #[test]
    fn it_can_be_interpolated() {
        let x = Quaternion::new_with(1f32, 0f32, 0f32, 0f32);
        let y = Quaternion::new_with(0f32, 0f32, 0f32, 1f32);

        let t1 = x.lerp(&y, 0f32);
        assert!((t1.v.x - x.v.x).abs() < 1e-6f32);
        assert!((t1.v.y - x.v.y).abs() < 1e-6f32);
        assert!((t1.v.z - x.v.z).abs() < 1e-6f32);
        assert!((t1.w - x.w).abs() < 1e-6f32);

        let t2 = x.lerp(&y, 1f32);
        assert!((t2.v.x - y.v.x).abs() < 1e-6f32);
        assert!((t2.v.y - y.v.y).abs() < 1e-6f32);
        assert!((t2.v.z - y.v.z).abs() < 1e-6f32);
        assert!((t2.w - y.w).abs() < 1e-6f32);
    }
}
