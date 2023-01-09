use bbox::BBox;
use bbox::Union;
use geometry::normal::Normal;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Cross;
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
        !(is_one(la2) && is_one(lb2) && is_one(lc2))
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
        let left = up.clone().normalize().into_cross_with(&dir).normalize();
        let new_up = dir.cross_with(&left);

        m[0][0] = left.x;
        m[1][0] = left.y;
        m[2][0] = left.z;
        m[3][0] = 0.0;

        m[0][1] = new_up.x;
        m[1][1] = new_up.y;
        m[2][1] = new_up.z;
        m[3][1] = 0.0;

        m[0][2] = dir.x;
        m[1][2] = dir.y;
        m[2][2] = dir.z;
        m[3][2] = 0.0;

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
    fn xf(&self, xf: T) -> T;
    fn t(&self, v: &T) -> T {
        self.xf(v.clone())
    }
}

impl ApplyTransform<Point> for Transform {
    fn xf(&self, p: Point) -> Point {
        let (x, y, z) = (p.x, p.y, p.z);
        let xt = self.m[0][0] * x + self.m[0][1] * y + self.m[0][2] * z + self.m[0][3];
        let yt = self.m[1][0] * x + self.m[1][1] * y + self.m[1][2] * z + self.m[1][3];
        let zt = self.m[2][0] * x + self.m[2][1] * y + self.m[2][2] * z + self.m[2][3];
        let w = self.m[3][0] * x + self.m[3][1] * y + self.m[3][2] * z + self.m[3][3];
        if w != 1f32 {
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
        let tx = self.xf(Vector::new_with(b.p_max.x - b.p_min.x, 0.0, 0.0));
        let ty = self.xf(Vector::new_with(0.0, b.p_max.y - b.p_min.y, 0.0));
        let tz = self.xf(Vector::new_with(0.0, 0.0, b.p_max.z - b.p_min.z));

        let tp = self.xf(b.p_min);
        BBox::from(tp.clone())
            .unioned_with(&tp + &tx)
            .unioned_with(&tp + &ty)
            .unioned_with(&tp + &tz)
            .unioned_with(&tp + &tx + &ty)
            .unioned_with(&tp + &tx + &tz)
            .unioned_with(&tp + &ty + &tz)
            .unioned_with(&tp + &tx + &ty + &tz)
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
        Transform::from(Matrix4x4::from(q))
    }
}

impl ::std::convert::From<Transform> for Quaternion {
    fn from(t: Transform) -> Quaternion {
        Quaternion::from(t.m)
    }
}

impl ::std::ops::Index<usize> for Transform {
    type Output = [f32; 4];
    fn index(&self, i: usize) -> &[f32; 4] {
        match i {
            0 ..= 3 => &self.m[i],
            _ => panic!("Error - Transform index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<usize> for Transform {
    fn index_mut(&mut self, i: usize) -> &mut [f32; 4] {
        match i {
            0 ..= 3 => &mut self.m[i],
            _ => panic!("Error - Transform index out of bounds!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbox::BBox;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use geometry::normal::Normal;
    use geometry::normal::Normalize;
    use quaternion::Quaternion;
    use ray::Ray;
    use ray::RayDifferential;
    use transform::matrix4x4::Matrix4x4;
    use utils::Degrees;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Transform::new(),
                   Transform {
                       m: Matrix4x4::new(),
                       m_inv: Matrix4x4::new()
                   });
    }

    #[test]
    fn it_can_be_created_with_matrices() {
        assert_eq!(Transform::new_with(Matrix4x4::new(), Matrix4x4::new()),
                   Transform::new());

        let m_ = Matrix4x4::new_with(2.0, 3.0,  1.0, 5.0,
                                     1.0, 0.0,  3.0, 1.0,
                                     0.0, 2.0, -3.0, 2.0,
                                     0.0, 2.0,  3.0, 1.0);
        let m_inv_ = m_.inverse();
        let xform = Transform::new_with(m_.clone(), m_inv_.clone());
        assert_eq!(xform, Transform { m: m_.clone(), m_inv: m_inv_ });
        assert_eq!(xform, Transform::from(m_));
        assert_eq!(xform, Transform::from([[2.0, 3.0,  1.0, 5.0],
                                           [1.0, 0.0,  3.0, 1.0],
                                           [0.0, 2.0, -3.0, 2.0],
                                           [0.0, 2.0,  3.0, 1.0]]));
    }

    #[test]
    fn it_can_transform_vectors() {
        let c = (45f32).as_radians().cos();
        let s = (45f32).as_radians().sin();

        let m_rot_x = Matrix4x4::new_with(1.0, 0.0, 0.0, 0.0,
                                          0.0, c, -s, 0.0,
                                          0.0, s, c, 0.0,
                                          0.0, 0.0, 0.0, 1.0);
        let v = Vector::new_with(1.0, 1.0, 0.0).normalize();
        let xform = Transform::new_with(m_rot_x.clone(), m_rot_x.inverse());

        let vt = Vector::new_with(2f32.sqrt() / 2.0, 0.5, 0.5);
        assert!((xform.t(&v) - &vt).length_squared() < 1e-6);
        assert!((xform.xf(v) - &vt).length_squared() < 1e-6);
        assert_eq!(vt, Transform::new().t(&vt));

        let xform2 = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
            * Transform::rotate_y(90.0);

        assert!((xform2.xf(Vector::new_with(1.0, 1.0, 1.0)) -
                Vector::new_with(1.0, 1.0, -1.0)).length_squared() < 1e-6);
        assert_eq!(xform2.xf(Vector::new()), Vector::new());
        assert_eq!(Transform::new().xf(Vector::new()), Vector::new());
        assert_eq!(Transform::new().xf(Vector::new_with(1.0, 5.0, -13.0)),
                   Vector::new_with(1.0, 5.0, -13.0));
    }

    #[test]
    fn it_cannot_translate_vectors() {
        let v = Vector::new_with(1.0, 1.0, 1.0);
        let xform = Transform::translate(&Vector::new_with(1.0, 4.0, -300.0));
        assert_eq!(xform.t(&v), v);
    }


    #[test]
    fn it_can_transform_points() {
        let xform = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
            * Transform::rotate_y(90.0);

        assert_eq!(xform.xf(Point::new_with(1.0, 1.0, 1.0)),
                   Point::new_with(2.0, 3.0, 2.0));
        assert_eq!(xform.xf(Point::new()), Point::new_with(1.0, 2.0, 3.0));
        assert_eq!(Transform::new().xf(Point::new()), Point::new());
        assert_eq!(Transform::new().xf(Point::new_with(1.0, 5.0, -13.0)),
                   Point::new_with(1.0, 5.0, -13.0));
    }

    #[test]
    fn it_can_transform_normals() {
        let xform = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
            * Transform::rotate_y(90.0);
        assert_eq!(xform.xf(Normal::new_with(1.0, 1.0, 1.0).normalize()),
                   Normal::new_with(1.0, 1.0, -1.0).normalize());
        assert_eq!(Transform::scale(2.0, 2.0, 2.0).xf(
            Normal::new_with(1.0, 0.0, 0.0)).normalize(),
                   Normal::new_with(1.0, 0.0, 0.0));
        assert_eq!(Transform::rotate_y(45.0).xf(
            Normal::new_with(1.0, 1.0, 1.0).normalize()).normalize(),
                   Normal::new_with(0.8164967, 0.5773503, 0.0));
    }

    #[test]
    fn it_can_be_inverted() {
        let p = Point::new_with(1.0, 2.0, 3.0);
        let m = Matrix4x4::new_with(2.0, 3.0,  1.0, 5.0,
                                    1.0, 0.0,  3.0, 1.0,
                                    0.0, 2.0, -3.0, 2.0,
                                    0.0, 2.0,  3.0, 1.0);
        let xform = Transform::new_with(m.clone(), m.invert());
        let xform_inv = xform.inverse();

        assert!((xform_inv.xf(xform.t(&p)) - p).length_squared() < 1e-6);
        assert_eq!(Transform::new(), Transform::new().invert());
    }

    #[test]
    fn it_can_translate_points() {
        let p = Point::new_with(1.0, 1.0, 1.0);
        let v = Vector::new_with(1.0, 2.0, 3.0);
        let xform = Transform::translate(&v);
        assert_eq!(xform.xf(p), Point::new_with(2.0, 3.0, 4.0));
    }

    #[test]
    fn it_can_scale_vectors() {
        let xform = Transform::scale(2.0, 0.5, 100.0);
        assert_eq!(xform.xf(Vector::new_with(1.0, 2.0, 0.0)), Vector::new_with(2.0, 1.0, 0.0));
        assert_eq!(xform.xf(Vector::new_with(-1.0, 0.0, -0.01)), Vector::new_with(-2.0, 0.0, -1.0));
    }

    #[test]
    fn it_knows_if_it_has_scale() {
        assert!(!Transform::new().has_scale());
        assert!(!Transform::rotate_x(3.0).has_scale());
        assert!(Transform::scale(2.0, 0.5, 100.0).has_scale());
        assert!(Transform::scale(2.0, 1.0, 100.0).has_scale());
        assert!(Transform::scale(2.0, 0.5, 1.0).has_scale());
        assert!(Transform::scale(1.0, 0.5, 100.0).has_scale());
        assert!(Transform::scale(0.0, 0.0, 0.0).has_scale());
    }

    #[test]
    fn it_can_rotate_about_x() {
        let xform = Transform::rotate_x(45.0);
        assert_eq!(xform.xf(Vector::new_with(1.0, 0.0, 0.0)),
                   Vector::new_with(1.0, 0.0, 0.0));
        assert_eq!(xform.xf(Vector::new()), Vector::new());
        assert!((xform.xf(Vector::new_with(1.0, 1.0, 0.0).normalize()) -
                 Vector::new_with(2f32.sqrt() / 2.0, 0.5, 0.5)).length_squared() < 1e-6);
    }

    #[test]
    fn it_can_rotate_about_y() {
        let xform = Transform::rotate_y(45.0);
        assert_eq!(xform.xf(Vector::new_with(0.0, 1.0, 0.0)),
                   Vector::new_with(0.0, 1.0, 0.0));
        assert_eq!(xform.xf(Vector::new()), Vector::new());
        assert!((xform.xf(Vector::new_with(1.0, 1.0, 0.0).normalize()) -
                 Vector::new_with(0.5, 2f32.sqrt() / 2.0, -0.5)).length_squared() < 1e-6);
    }

    #[test]
    fn it_can_rotate_about_z() {
        let xform = Transform::rotate_z(45.0);
        assert_eq!(xform.xf(Vector::new_with(0.0, 0.0, 1.0)),
                   Vector::new_with(0.0, 0.0, 1.0));
        assert_eq!(xform.xf(Vector::new()), Vector::new());
        assert!((xform.xf(Vector::new_with(0.0, 1.0, 1.0).normalize()) -
                 Vector::new_with(-0.5, 0.5, 2f32.sqrt() / 2.0)).length_squared() < 1e-6);
    }

    #[test]
    fn it_can_rotate_about_arbitrary_axes() {
        let x_axis = Vector::new_with(1.0, 0.0, 0.0);
        let xform = Transform::rotate(45.0, &x_axis);

        assert_eq!(xform.t(&x_axis), x_axis);
        assert_eq!(xform.xf(Vector::new()), Vector::new());
        assert!((xform.xf(Vector::new_with(1.0, 1.0, 0.0).normalize()) -
                 Vector::new_with(2f32.sqrt() / 2.0, 0.5, 0.5)).length_squared() < 1e-6);

        let xform2 = Transform::rotate(120.0, &Vector::new_with(1.0, 1.0, 1.0));
        assert!((xform2.xf(Vector::new_with(1.0, 0.0, 0.0)) -
                 Vector::new_with(0.0, 1.0, 0.0)).length_squared() < 1e-6);
        assert!((xform2.xf(Vector::new_with(0.0, 1.0, 0.0)) -
                 Vector::new_with(0.0, 0.0, 1.0)).length_squared() < 1e-6);
        assert!((xform2.xf(Vector::new_with(0.0, 0.0, 1.0)) -
                 Vector::new_with(1.0, 0.0, 0.0)).length_squared() < 1e-6);
    }

    #[test]
    fn it_can_look_at_a_point() {
        let origin = Point::new();
        let pos = Point::new_with(1.0, 1.0, 1.0);
        let up = Vector::new_with(0.0, 1.0, 0.0);

        let xform = Transform::look_at(&pos, &origin, &up);
        assert!(xform.t(&origin).x.abs() < 1e-6);
        assert!(xform.t(&origin).y.abs() < 1e-6);
        assert!(xform.t(&origin).z > 1.0);

        assert!(xform.xf(Point::new_with(-1.0, -1.0, -1.0)).x.abs() < 1e-6);
        assert!(xform.xf(Point::new_with(-1.0, -1.0, -1.0)).y.abs() < 1e-6);
        assert!(xform.xf(Point::new_with(-1.0, -1.0, -1.0)).z > 2.0);

        let pos2 = Point::new_with(1.0, 2.0, 3.0);
        let look = Point::new_with(-1.0, 0.0, 4.0);

        let xform2 = Transform::look_at(&pos2, &look, &up);

        // The x-y plane for this transform should have the plane equation:
        // (-2/3)A + (-2/3)B + (1/3)C + 1 = 0
        // We want the intersection with the Z-axis, this should be whatever
        // C value satisfies the plane for (0, 0, C), so C = -3.
        assert_eq!(xform2.xf(Point::new_with(0.0, 0.0, -3.0)).z, 0.0);

        // Because the direction of the camera should be in the negative x
        // and y direction, the resulting point should have a positive x/y
        // coordinate...
        assert!(xform2.xf(Point::new_with(0.0, 0.0, -3.0)).x.abs() > 0.1);
        assert!(xform2.xf(Point::new_with(0.0, 0.0, -3.0)).y.abs() > 0.1);

        // The midpoint between the two points should also be along the z axis.
        assert!(xform2.xf(Point::new_with(0.0, 1.0, 3.5)).z > 1.0);
        assert!(xform2.xf(Point::new_with(0.0, 1.0, 3.5)).x.abs() < 1e-6);
        assert!(xform2.xf(Point::new_with(0.0, 1.0, 3.5)).y.abs() < 1e-6);
    }

    #[test]
    fn it_can_detect_handedness_swap() {
        let m = Matrix4x4::new();

        // If any of the axes are swapped, then this matrix should "swap handedness"
        assert!(!Transform::new_with(m.clone(), m.clone()).swaps_handedness());

        let mut m2 = m.clone();

        m2[0][0] = -1.0;
        assert!(Transform::new_with(m2.clone(), m2.inverse()).swaps_handedness());
        m2[0][0] = 1.0;
        m2[1][1] = -1.0;
        assert!(Transform::new_with(m2.clone(), m2.inverse()).swaps_handedness());
        m2[1][1] = 1.0;
        m2[2][2] = -1.0;
        assert!(Transform::new_with(m2.clone(), m2.inverse()).swaps_handedness());

        let xform = Transform::new_with(m2.clone(), m2.inverse()) *
            Transform::rotate_x(34.0);
        let xform2 = &xform * Transform::translate(&Vector::new_with(1.0, -2.0, 4.0));
        let xform3 = &xform2 * Transform::look_at((&Point::new_with(1.0, 0.0, -3.0)),
                                                  (&Point::new_with(15.0, 12.0, -0.0)),
                                                  (&Vector::new_with(0.0, 1.0, 0.0)));

        // If one matrix swaps handedness, it infects all of the subsequent ones...
        assert!(xform.swaps_handedness());
        assert!(xform2.swaps_handedness());
        assert!(xform3.swaps_handedness());

        // But we can swap it back...
        assert!(!(xform3 * Transform::new_with(m2.clone(), m2.inverse()))
                .swaps_handedness());
    }

    #[test]
    fn it_can_transform_rays() {
        let r = Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                              Vector::new_with(1.0, 0.0, 0.0), 0.0);
        let rd = RayDifferential::new_with(Point::new_with(0.0, 0.0, 0.0),
                                           Vector::new_with(1.0, 0.0, 0.0), 0.0);
        assert_eq!(r, Transform::new().t(&r));
        assert_eq!(rd, Transform::new().t(&rd));

        let xform =
            Transform::translate(&Vector::new_with(1.0, 1.0, 1.0)) *
            Transform::rotate_y(90.0);

        let tr = xform.t(&r);
        let trd = xform.t(&rd);
        let tr_expected = Ray::new_with(
            Point::new_with(1.0, 1.0, 1.0), Vector::new_with(0.0, 0.0, -1.0), 0.0);
        let trd_expected = RayDifferential::new_with(
            Point::new_with(1.0, 1.0, 1.0), Vector::new_with(0.0, 0.0, -1.0), 0.0);

        assert_eq!(tr.o, tr_expected.o);
        assert!((&tr.d - &tr_expected.d).length_squared() < 1e-6);
        assert_eq!(tr.mint(), tr_expected.mint());
        assert_eq!(tr.maxt(), tr_expected.maxt());
        assert_eq!(tr.time, tr_expected.time);
        assert_eq!(tr.depth, tr_expected.depth);

        assert_eq!(trd.ray.o, trd_expected.ray.o);
        assert!((&trd.ray.d - &trd_expected.ray.d).length_squared() < 1e-6);
        assert_eq!(trd.ray.mint(), trd_expected.ray.mint());
        assert_eq!(trd.ray.maxt(), trd_expected.ray.maxt());
        assert_eq!(trd.ray.time, trd_expected.ray.time);
        assert_eq!(trd.ray.depth, trd_expected.ray.depth);
    }

    #[test]
    fn it_can_transform_bboxes() {
        let bbox = BBox::new_with(Point::new_with(-1.0, -1.0, -1.0),
                                  Point::new_with(1.0, 1.0, 1.0));

        assert_eq!(bbox, Transform::new().t(&bbox));

        let xform =
            Transform::translate(&Vector::new_with(-3.0, 0.0, 1.0)) *
            Transform::rotate_x(45.0);

        // If you rotate (1, 1, 1) 45 degrees about X, you get a
        // vector of the same length in the X-Z plane with x = 1:
        // sqrt(3) = sqrt(1 + z^2) sooo.. z = sqrt(2)
        // This means our extents are sqrt(2) along y and z, but not X...

        let expected_bbox_max =
            Point::new_with(-2.0, 2f32.sqrt(), 2f32.sqrt() + 1.0);
        let expected_bbox_min =
            Point::new_with(-4.0, -(2f32.sqrt()), -(2f32.sqrt()) + 1.0);
        let expected_bbox = BBox::new_with(expected_bbox_min, expected_bbox_max);

        assert_eq!(xform.t(&bbox), expected_bbox);
    }

    #[test]
    fn it_can_transform_to_and_from_quaternions() {
        let id = Quaternion::new();
        assert_eq!(id, Quaternion::from(Transform::from(id.clone())));

        // Random-ish quaternion?
        let q = Quaternion::new_with(1.0, 4.0, 16.0, 2.0).normalize();
        assert_eq!(q, Quaternion::from(Transform::from(q.clone())));
    }
}
