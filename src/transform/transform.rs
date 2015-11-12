use bbox::BBox;
use bbox::Union;
use geometry::normal::Normal;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Dot;
use geometry::vector::Vector;
use quaternion::Quaternion;
use ray::Ray;
use ray::RayDifferential;
use transform::matrix4x4::Matrix4x4;
use utils::Degrees;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Transform {
    // Transform private data
    m: Matrix4x4,
    m_inv: Matrix4x4
}

impl Transform {
    // Transform public methods
    pub fn new() -> Transform {
        Transform { m: Matrix4x4::new(), m_inv: Matrix4x4::new() }
    }

    pub fn new_with(_m: Matrix4x4, _inv: Matrix4x4) -> Transform {
        Transform { m: _m, m_inv: _inv }
    }

    pub fn invert(self) -> Transform {
        Transform::new_with(self.m_inv, self.m)
    }

    pub fn inverse(&self) -> Transform {
        self.clone().invert()
    }

    pub fn translate(v: &Vector) -> Transform {
        let m = Matrix4x4::new_with(
            1f32, 0f32, 0f32, v.x,
            0f32, 1f32, 0f32, v.y,
            0f32, 0f32, 1f32, v.z,
            0f32, 0f32, 0f32, 1f32);
        let m_inv = Matrix4x4::new_with(
            1f32, 0f32, 0f32, -v.x,
            0f32, 1f32, 0f32, -v.y,
            0f32, 0f32, 1f32, -v.z,
            0f32, 0f32, 0f32, 1f32);
        Transform::new_with(m, m_inv)
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Transform {
        let m = Matrix4x4::new_with(
            x, 0f32, 0f32, 0f32,
            0f32, y, 0f32, 0f32,
            0f32, 0f32, z, 0f32,
            0f32, 0f32, 0f32, 1f32);
        let m_inv = Matrix4x4::new_with(
            (1f32/x), 0f32, 0f32, 0f32,
            0f32, (1f32/y), 0f32, 0f32,
            0f32, 0f32, (1f32/z), 0f32,
            0f32, 0f32, 0f32, 1f32);
        Transform::new_with(m, m_inv)
    }

    pub fn has_scale(&self) -> bool {
        let la2 = Vector::new_with(self.m[0][0],
                                   self.m[1][0],
                                   self.m[2][0]).length_squared();
        let lb2 = Vector::new_with(self.m[0][1],
                                   self.m[1][1],
                                   self.m[2][1]).length_squared();
        let lc2 = Vector::new_with(self.m[0][2],
                                   self.m[1][2],
                                   self.m[2][2]).length_squared();
        let is_one = |x| x > 0.999 && x < 1.001;
        is_one(la2) && is_one(lb2) && is_one(lc2)
    }

    pub fn rotate_x(angle: f32) -> Transform {
        let sin_t = angle.as_radians().sin();
        let cos_t = angle.as_radians().cos();
        let m = Matrix4x4::new_with(
            1f32, 0f32, 0f32, 0f32,
            0f32, cos_t, -sin_t, 0f32,
            0f32, sin_t, cos_t, 0f32,
            0f32, 0f32, 0f32, 1f32);
        let m_inv = m.clone().transpose();
        Transform::new_with(m, m_inv)
    }

    pub fn rotate_y(angle: f32) -> Transform {
        let sin_t = angle.as_radians().sin();
        let cos_t = angle.as_radians().cos();
        let m = Matrix4x4::new_with(
            cos_t, 0f32, sin_t, 0f32,
            0f32, 1f32, 0f32, 0f32,
            -sin_t, 0f32, cos_t, 0f32,
            0f32, 0f32, 0f32, 1f32);
        let m_inv = m.clone().transpose();
        Transform::new_with(m, m_inv)
    }

    pub fn rotate_z(angle: f32) -> Transform {
        let sin_t = angle.as_radians().sin();
        let cos_t = angle.as_radians().cos();
        let m = Matrix4x4::new_with(
            cos_t, -sin_t, 0f32, 0f32,
            sin_t, cos_t, 0f32, 0f32,
            0f32, 0f32, 1f32, 0f32,
            0f32, 0f32, 0f32, 1f32);
        let m_inv = m.clone().transpose();
        Transform::new_with(m, m_inv)
    }

    pub fn rotate(angle: f32, axis: &Vector) -> Transform {
        let a: Vector = axis.clone().normalize();
        let s = angle.as_radians().sin();
        let c = angle.as_radians().cos();

        let mut m = Matrix4x4::new();
        m[0][0] = a.x * a.x + (1f32 - a.x * a.x) * c;
        m[0][1] = a.x * a.y * (1f32 - c) - a.z * s;
        m[0][2] = a.x * a.z * (1f32 - c) + a.y * s;
        m[0][3] = 0f32;

        m[1][0] = a.x * a.y * (1f32 - c) + a.z * s;
        m[1][1] = a.y * a.y + (1f32 - a.y * a.y) * c;
        m[1][2] = a.y * a.z * (1f32 - c) - a.x * s;
        m[1][3] = 0f32;

        m[2][0] = a.x * a.z * (1f32 - c) - a.y * s;
        m[2][1] = a.y * a.z * (1f32 - c) + a.x * s;
        m[2][2] = a.z * a.z + (1f32 - a.z * a.z) * c;
        m[2][3] = 0f32;

        m[3] = [0f32, 0f32, 0f32, 1f32];

        let m_inv = m.clone().transpose();
        Transform::new_with(m, m_inv)
    }

    pub fn look_at(pos: &Point, look: &Point, up: &Vector) -> Transform {
        let mut m = Matrix4x4::new();

        // Initialize fourth column of viewing matrix
        m[0][3] = pos.x;
        m[1][3] = pos.y;
        m[2][3] = pos.z;
        m[3][3] = 1f32;

        // Initialize first three columns of viewing matrix
        let dir = (look - pos).normalize();
        let up_norm = up.clone().normalize();
        let left = up_norm.cross(&dir).normalize();
        let new_up = dir.clone().cross(&left);

        m[0][0] = left.x;
        m[1][0] = left.y;
        m[2][0] = left.z;
        m[3][0] = 0f32;

        m[0][1] = new_up.x;
        m[1][1] = new_up.y;
        m[2][1] = new_up.z;
        m[3][1] = 0f32;

        m[0][2] = dir.x;
        m[1][2] = dir.y;
        m[2][2] = dir.z;
        m[3][2] = 0f32;

        Transform::new_with(m.clone().invert(), m)
    }

    pub fn swaps_handedness(&self) -> bool {
        0f32 > (self.m[0][0] * (self.m[1][1] * self.m[2][2] -
                                self.m[1][2] * self.m[2][1]) -
                self.m[0][1] * (self.m[1][0] * self.m[2][2] -
                                self.m[1][2] * self.m[2][0]) +
                self.m[0][2] * (self.m[1][0] * self.m[2][1] -
                                self.m[1][1] * self.m[2][0]))
    }

    pub fn get_matrix<'a>(&'a self) -> &'a Matrix4x4 { &(self.m) }
}

pub trait ApplyTransform<T : Clone> {
    fn xf(&self, T) -> T;
    fn t(&self, v: &T) -> T {
        self.xf(v.clone())
    }
}

impl ApplyTransform<Point> for Transform {
    fn xf(&self, p: Point) -> Point {
        let (x, y, z) = (p.x, p.y, p.z);
        let xt = self.m[0][0] * x + self.m[0][1] * y +
            self.m[0][2] * z + self.m[0][3];
        let yt = self.m[1][0] * x + self.m[1][1] * y + self.m[1][2] * z + self.m[1][3];
        let zt = self.m[2][0] * x + self.m[2][1] * y + self.m[2][2] * z + self.m[2][3];
        let w = self.m[3][0] * x + self.m[3][1] * y + self.m[3][2] * z + self.m[3][3];
        if (w != 1f32) {
            Point::new_with(xt / w, yt / w, zt / w)
        } else {
            Point::new_with(xt, yt, zt)
        }
    }
}

impl ApplyTransform<Vector> for Transform {
    fn xf(&self, p: Vector) -> Vector {
        let (x, y, z) = (p.x, p.y, p.z);
        let xt = self.m[0][0] * x + self.m[0][1] * y + self.m[0][2] * z;
        let yt = self.m[1][0] * x + self.m[1][1] * y + self.m[1][2] * z;
        let zt = self.m[2][0] * x + self.m[2][1] * y + self.m[2][2] * z;
        Vector::new_with(xt, yt, zt)
    }
}

impl ApplyTransform<Normal> for Transform {
    fn xf(&self, n: Normal) -> Normal {
        let (x, y, z) = (n.x, n.y, n.z);
        let xt = self.m_inv[0][0] * x + self.m_inv[1][0] * y + self.m_inv[2][0] * z;
        let yt = self.m_inv[0][1] * x + self.m_inv[1][1] * y + self.m_inv[2][1] * z;
        let zt = self.m_inv[0][2] * x + self.m_inv[1][2] * y + self.m_inv[2][2] * z;
        Normal::new_with(xt, yt, zt)
    }
}

impl ApplyTransform<Ray> for Transform {
    fn xf(&self, r: Ray) -> Ray {
        let mut ret = r.clone();
        ret.o = self.t(&r.o);
        ret.d = self.t(&r.d);
        ret
    }
}

impl ApplyTransform<RayDifferential> for Transform {
    fn xf(&self, r: RayDifferential) -> RayDifferential {
        let mut ret = r.clone();
        ret.ray.o = self.t(&r.ray.o);
        ret.ray.d = self.t(&r.ray.d);
        ret
    }
}

impl ApplyTransform<BBox> for Transform {
    fn xf(&self, b: BBox) -> BBox {
        BBox::new().unioned_with(
            &self.xf(Point::new_with(b.p_min.x, b.p_min.y, b.p_min.z))).unioned_with(
            &self.xf(Point::new_with(b.p_min.x, b.p_min.y, b.p_max.z))).unioned_with(
            &self.xf(Point::new_with(b.p_min.x, b.p_max.y, b.p_min.z))).unioned_with(
            &self.xf(Point::new_with(b.p_min.x, b.p_max.y, b.p_max.z))).unioned_with(
            &self.xf(Point::new_with(b.p_max.x, b.p_min.y, b.p_min.z))).unioned_with(
            &self.xf(Point::new_with(b.p_max.x, b.p_min.y, b.p_max.z))).unioned_with(
            &self.xf(Point::new_with(b.p_max.x, b.p_max.y, b.p_min.z))).unioned_with(
            &self.xf(Point::new_with(b.p_max.x, b.p_max.y, b.p_max.z)))
    }
}

impl<'a, 'b> ::std::ops::Mul<&'a Transform> for &'b Transform {
    type Output = Transform;
    fn mul(self, t: &'a Transform) -> Transform {
        Transform::new_with(&self.m * &t.m, &t.m_inv * &self.m_inv)
    }
}

impl<'a> ::std::ops::Mul<Transform> for &'a Transform {
    type Output = Transform;
    fn mul(self, t: Transform) -> Transform { self * &t }
}

impl<'a> ::std::ops::Mul<&'a Transform> for Transform {
    type Output = Transform;
    fn mul(self, t: &'a Transform) -> Transform { &self * t }
}

impl ::std::ops::Mul<Transform> for Transform {
    type Output = Transform;
    fn mul(self, t: Transform) -> Transform { &self * &t }
}

impl ::std::convert::From<Matrix4x4> for Transform {
    fn from(m: Matrix4x4) -> Transform {
        let inv = m.inverse();
        Transform::new_with(m, inv)
    }
}

impl ::std::convert::From<[[f32; 4]; 4]> for Transform {
    fn from(mat: [[f32; 4]; 4]) -> Transform {
        Transform::from(Matrix4x4::from(mat))
    }
}

impl ::std::convert::From<Quaternion> for Transform {
    fn from(q: Quaternion) -> Transform {
        let x = q.v.x;
        let y = q.v.y;
        let z = q.v.z;
        let w = q.w;

        debug_assert!((q.dot(&q).sqrt() - 1f32).abs() < 1e-4f32,
                      "Quaternion must be unit before conversion to Transform");
        Transform::from([
            [1f32 - 2f32*(y*y+z*z), 2f32*(x*y+z*w), 2f32*(x*z-y*w), 0f32],
            [2f32*(x*y-z*w), 1f32 - 2f32*(x*x+z*z), 2f32*(y*z+x*w), 0f32],
            [2f32*(x*z+y*w), 2f32*(y*z-x*w), 1f32 - 2f32*(x*x+y*y), 0f32],
            [0f32, 0f32, 0f32, 1f32]])
    }
}

impl ::std::convert::From<Transform> for Quaternion {
    fn from(t: Transform) -> Quaternion {
        Quaternion::from(t.m)
    }
}
