use geometry::Dot;
use geometry::Lerp;
use geometry::Normalize;
use geometry::Vector;

use utils::Clamp;

pub struct Quaternion {
    pub v: Vector,
    pub w: f32
}

impl Quaternion {
    pub fn new() -> Quaternion {
        Quaternion {
            v: Vector::new(),
            w: 1f32
        }
    }

    pub fn new_with(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion {
            v: Vector::new_with(x, y, z),
            w: w
        }
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

impl Lerp for Quaternion {
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
