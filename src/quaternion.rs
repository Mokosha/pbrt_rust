use geometry::normal::Normalize;
use geometry::vector::Dot;
use geometry::vector::Vector;
use std::f32;
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
        if (cos_theta < 0f32) {
            self.lerp(&(-q), t)
        } else if (cos_theta > 0.9995f32) {
            ((1f32 - t) * self + t * q).normalize()
        } else if (cos_theta < 1e-6) {
            // In this case the quaternions are perpendicular
            // in rotational space, so we don't need to do an expensive
            // acos...
            let thetap = (f32::consts::PI / 2.0) * t;
            self * thetap.cos() + q * thetap.sin()
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
        Quaternion::new_with(-(self.v.x), -(self.v.y), -(self.v.z), -(self.w))
    }
}

impl ::std::ops::Neg for Quaternion {
    type Output = Quaternion;
    fn neg(self) -> Quaternion { -(&self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::normal::Normalize;
    use geometry::vector::Vector;
    use geometry::vector::Dot;
    use std::f32;
    use utils::Lerp;
    use utils::Clamp;

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
    fn it_can_be_normalized() {
        let ca = (f32::consts::PI * 0.25f32).cos();
        let sa = (f32::consts::PI * 0.25f32).sin();
        {
            let q = Quaternion::new_with(0f32 * sa, 0f32 * sa, 1f32 * sa, 1f32 * ca).normalize();
            assert!((q.dot(&q).sqrt() - 1f32).abs() < 1e-6f32);
        }
        {
            let q = Quaternion::new_with(0f32 * sa, 0f32 * sa, 2f32 * sa, 1f32 * ca).normalize();
            assert!((q.dot(&q).sqrt() - 1f32).abs() < 1e-6f32);
        }
        {
            let q = Quaternion::new_with(1f32 * sa, 2f32 * sa, 3f32 * sa, 1f32 * ca).normalize();
            assert!((q.dot(&q).sqrt() - 1f32).abs() < 1e-6f32);
        }
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

    // Shamelessly grab these tests from the glm tests:
    // https://github.com/g-truc/glm/blob/master/test/gtc/gtc_quaternion.cpp#L163
    #[test]
    fn it_can_be_interpolated() {
        macro_rules! check_quat {
            ($q1: expr, $q2: expr) => {{
                let u = ($q1).clone();
                let v = ($q2).clone();
                if (u.dot(&v).powi(2) - 1.0).abs() >= 1e-6 {
                    println!("");
                    println!("q1: {:?}", u);
                    println!("q2: {:?}", v);
                    panic!();
                }
            }}
        };

        let sqrt2 = 2f32.sqrt() / 2f32;
        let id = Quaternion::new();
        let y90rot = Quaternion::new_with(0.0f32, sqrt2, 0.0f32, sqrt2);

        // Testing a == 0
        // Must be id
        {
            let id2 = id.lerp(&y90rot, 0.0f32);
            check_quat!(&id2, &id);
        }

        // Testing a == 1
        // Must be 90° rotation on Y : 0 0.7 0 0.7
        {
            let y90rot2 = id.lerp(&y90rot, 1f32);
            check_quat!(&y90rot2, &y90rot);
        }

        // Testing standard, easy case
        // Must be 45° rotation on Y : 0 0.38 0 0.92
        let y45rot1 = id.lerp(&y90rot, 0.5f32);

        {
            // Testing reverse case
            // Must be 45° rotation on Y : 0 0.38 0 0.92
            let ym45rot2 = y90rot.lerp(&id, 0.5f32);
            check_quat!(&y45rot1, &ym45rot2);

            // Testing against full circle around the sphere instead of shortest path
            // Must be 45° rotation on Y
            // certainly not a 135° rotation
            let y45rot3 = id.lerp(&(-(&y90rot)), 0.5f32);
            check_quat!(&ym45rot2, &y45rot3);

            // Same, but inverted
            // Must also be 45° rotation on Y :  0 0.38 0 0.92
            // -0 -0.38 -0 -0.92 is ok too
            let y45rot4 = (-(&y90rot)).lerp(&id, 0.5f32);
            check_quat!(&ym45rot2, &-(&y45rot4));
        }

        {
            // Testing q1 = q2
            // Must be 90° rotation on Y : 0 0.7 0 0.7
            let y90rot3 = y90rot.lerp(&y90rot, 0.5f32);
            check_quat!(&y90rot, &y90rot3);
        }

        // Testing quaternions with opposite sign
        {
            let a = Quaternion::new_with(0f32, 0f32, 0f32, -1f32);
            let result = a.lerp(&id, 0.5f32);
            assert_eq!(id.dot(&result).powi(2), 1f32);
        }

        // Testing non 0.5 slerp
        {
            let q1 = Quaternion::new_with(0.0, 1.0, 0.0, 0.0);
            let q2 = Quaternion::new_with(0.0, 0.0, 0.0, 1.0);
            let pi_8 = f32::consts::PI / 8.0;
            let result = Quaternion::new_with(0.0, 1.0 * pi_8.sin(), 0.0,
                                              pi_8.cos());
            let result_a = Quaternion::new_with(0.0, -1.0 * pi_8.sin(), 0.0,
                                                pi_8.cos());

            check_quat!(q2.lerp(&q1, 0.25), result);
            check_quat!(q2.lerp(&(-(&q1)), 0.25), result_a);
            check_quat!(-(&q2).lerp(&q1, 0.25), result);
            check_quat!(-(&q2).lerp(&(-(&q1)), 0.25), result_a);
        }
    }
}
