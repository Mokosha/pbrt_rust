use geometry::Dot;
use geometry::Normalize;
use geometry::Vector;

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
