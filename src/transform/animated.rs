use bbox::BBox;
use bbox::Union;
use geometry::point::Point;
use geometry::vector::Vector;
use quaternion::Quaternion;
use ray::Ray;
use ray::RayDifferential;
use transform::matrix4x4::Matrix4x4;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Lerp;

macro_rules! check_mat {
    ($m1: expr, $m2: expr) => {{
        let x = ($m1).clone();
        let y = ($m2).clone();
        for i in 0..4 {
            for j in 0..4 {
                let diff = (x[i][j] - y[i][j]).abs();
                if diff >= 5e-5 {
                    println!("m1: {:?}", x);
                    println!("m2: {:?}", y);
                    println!("Matrices differ at {:?} by {:?}", (i, j), diff);
                    panic!();
                }
            }
        }
    }}
}

macro_rules! check_animated_xform {
    ($q1: expr, $q2: expr) => {{
        let xf1 = &(($q1).clone());
        let xf2 = &(($q2).clone());

        if (xf1.start_time - xf2.start_time).abs() >= 1e-6 {
            println!("Animated transforms differ on start_time");
            println!("xform1: {:?}", xf1.start_time);
            println!("xform2: {:?}", xf2.start_time);
            panic!();
        }

        if (xf1.end_time - xf2.end_time).abs() >= 1e-6 {
            println!("Animated transforms differ on end_time");
            println!("xform1: {:?}", xf1.end_time);
            println!("xform2: {:?}", xf2.end_time);
            panic!();
        }

        if xf1.actually_animated != xf2.actually_animated {
            println!("Animated transforms differ on actually_animated");
            println!("xform1: {:?}", xf1.actually_animated);
            println!("xform2: {:?}", xf2.actually_animated);
            panic!();
        }

        if (xf1.t1.clone() - xf2.t1.clone()).length_squared() >= 1e-6 {
            println!("Animated transforms differ on t1");
            println!("xform1: {:?}", xf1.t1);
            println!("xform2: {:?}", xf2.t1);
            panic!();
        }

        if (xf1.t2.clone() - xf2.t2.clone()).length_squared() >= 1e-6 {
            println!("Animated transforms differ on t2");
            println!("xform1: {:?}", xf1.t2);
            println!("xform2: {:?}", xf2.t2);
            panic!();
        }

        if (xf1.r1.dot(&xf2.r1).powi(2) - 1.0).abs() >= 1e-6 {
            println!("Animated transforms differ on r1");
            println!("xform1: {:?}", xf1.r1);
            println!("xform2: {:?}", xf2.r1);
            panic!();
        }

        if (xf1.r2.dot(&xf2.r2).powi(2) - 1.0).abs() >= 1e-6 {
            println!("Animated transforms differ on r2");
            println!("xform1: {:?}", xf1.r2);
            println!("xform2: {:?}", xf2.r2);
            panic!();
        }

        check_mat!(xf1.s1, xf2.s1);
        check_mat!(xf1.s2, xf2.s2);
    }}
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnimatedTransform {
    start_time: f32,
    end_time: f32,
    start_transform: Transform,
    end_transform: Transform,
    actually_animated: bool,
    t1: Vector, t2: Vector,
    r1: Quaternion, r2: Quaternion,
    s1: Matrix4x4, s2: Matrix4x4
}

impl AnimatedTransform {
    pub fn new(transform1: Transform, time1: f32,
               transform2: Transform, time2: f32) -> AnimatedTransform {
        let (t1, r1, s1) = AnimatedTransform::decompose(&transform1);
        let (t2, r2, s2) = AnimatedTransform::decompose(&transform2);
        let animated = transform1.ne(&transform2);
        AnimatedTransform {
            start_time: time1,
            end_time: time2,
            start_transform: transform1,
            end_transform: transform2,
            actually_animated: animated,
            t1: t1, t2: t2,
            r1: r1, r2: r2,
            s1: s1, s2: s2
        }
    }

    fn interpolate(&self, time: f32) -> Transform {
        // Handle boundary conditions for matrix interpolation
        if (!self.actually_animated || time <= self.start_time) {
            return self.start_transform.clone();
        }

        if (time >= self.end_time) {
            return self.end_transform.clone();
        }

        let dt = (time - self.start_time) / (self.end_time - self.start_time);

        // Interpolate translation at dt
        let trans = self.t1.lerp(&self.t2, dt);

        // Interpolate rotation at dt
        let rotate = self.r1.lerp(&self.r2, dt);

        // Interpolate scale at dt
        let scale = self.s1.lerp(&self.s2, dt);

        // Compute interpolated matrix as product of interpolated components
        Transform::translate(&trans) *
            Transform::from(rotate) *
            Transform::from(scale)
    }

    fn decompose(transform: &Transform) -> (Vector, Quaternion, Matrix4x4) {
        let tm = transform.get_matrix();

        // Extract translation T from the transformation matrix
        let t = Vector::new_with(tm[0][3], tm[1][3], tm[2][3]);

        // Compute new transformation matrix M without translation
        let mut m = tm.clone();
        {
            // Scope the borrow of m...
            let m_ref = &mut m;
            for i in 0..3 {
                m_ref[i][3] = 0f32;
            }
            m_ref[3] = [0f32, 0f32, 0f32, 1f32];
        };

        // Extract rotation R from transformation matrix
        let mut r = m.clone();
        for _ in 0..100 {
            // Compute next matrix r_next in series
            let r_next = 0.5 * (&r + r.clone().invert().transpose());

            // Compute norm of difference between r and r_next
            let norm = (0..3).fold(0f32, |acc, i| {
                let r_ref = &r;
                let r_next_ref = &r_next;
                acc.max((r_ref[i][0] - r_next_ref[i][0]).abs() +
                        (r_ref[i][1] - r_next_ref[i][1]).abs() +
                        (r_ref[i][2] - r_next_ref[i][2]).abs())
            });

            if (norm < 0.0001f32) {
                break;
            }

            r = r_next;
        }

        // Compute scale S using rotation and original matrix
        let s = r.inverse() * m;
        (t, Quaternion::from(r), s)
    }

    pub fn motion_bounds(&self, b: &BBox, use_inverse: bool) -> BBox {
        if (!self.actually_animated) {
            return self.start_transform.inverse().t(b);
        }

        let num_steps = 128;
        (0..num_steps).fold(BBox::new(), |bbox, i| {
            let t = self.start_time.lerp(&self.end_time,
                                         ((i as f32) / ((num_steps - 1) as f32)));
            bbox.unioned_with(&(
                if (use_inverse) {
                    self.interpolate(t).invert().t(b)
                } else {
                    self.interpolate(t).t(b)
                }))
        })
    }

    pub fn xfpt(&self, time: f32, p: Point) -> Point {
        self.interpolate(time).xf(p)
    }

    pub fn tpt(&self, time: f32, p: &Point) -> Point {
        self.xfpt(time, p.clone())
    }

    pub fn xfvec(&self, time: f32, v: Vector) -> Vector {
        self.interpolate(time).xf(v)
    }

    pub fn tvec(&self, time: f32, v: &Vector) -> Vector {
        self.xfvec(time, v.clone())
    }
}

impl ApplyTransform<Ray> for AnimatedTransform {
    fn xf(&self, r: Ray) -> Ray {
        let mut ret = r.clone();
        let t = f32::from(r.time);
        ret.o = self.tpt(t, &ret.o);
        ret.d = self.tvec(t, &ret.d);
        ret
    }
}

impl ApplyTransform<RayDifferential> for AnimatedTransform {
    fn xf(&self, r: RayDifferential) -> RayDifferential {
        let mut ret = r.clone();
        let t = f32::from(r.ray.time);
        ret.ray.o = self.tpt(t, &ret.ray.o);
        ret.ray.d = self.tvec(t, &ret.ray.d);
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbox::BBox;
    use geometry::point::Point;
    use geometry::vector::Dot;
    use geometry::vector::Vector;
    use quaternion::Quaternion;
    use transform::matrix4x4::Matrix4x4;
    use transform::transform::Transform;
    use transform::transform::ApplyTransform;

    #[test]
    fn it_can_be_created() {
        let from = Transform::new();
        let mut expected_anim = AnimatedTransform {
            start_time: 0.0,
            end_time: 0.0,
            start_transform: from.clone(),
            end_transform: from.clone(),
            actually_animated: false,
            t1: Vector::new(), t2: Vector::new(),
            r1: Quaternion::new(), r2: Quaternion::new(),
            s1: Matrix4x4::new(), s2: Matrix4x4::new()
        };

        assert_eq!(expected_anim,
                   AnimatedTransform::new(from.clone(), 0.0, from.clone(), 0.0));

         expected_anim = AnimatedTransform {
            start_time: 0.0,
            end_time: 1.0,
            start_transform: from.clone(),
            end_transform: from.clone(),
            actually_animated: false,
            t1: Vector::new(), t2: Vector::new(),
            r1: Quaternion::new(), r2: Quaternion::new(),
            s1: Matrix4x4::new(), s2: Matrix4x4::new()
        };

        assert_eq!(expected_anim,
                   AnimatedTransform::new(from.clone(), 0.0, from.clone(), 1.0));

        let to = Transform::translate(&Vector::new_with(1.0, 1.0, 1.0)) *
            Transform::rotate_y(45.0);

         expected_anim = AnimatedTransform {
             start_time: 0.0,
             end_time: 1.0,
             start_transform: from.clone(),
             end_transform: to.clone(),
             actually_animated: true,
             t1: Vector::new(), t2: Vector::new_with(1.0, 1.0, 1.0),
             r1: Quaternion::new(),
             r2: Quaternion::new_with(0.0, 0.38268343236, 0.0, 0.92387953251),
             s1: Matrix4x4::new(), s2: Matrix4x4::new()
        };

        check_animated_xform!(
            expected_anim,
            AnimatedTransform::new(from.clone(), 0.0, to.clone(), 1.0));
    }

    #[test]
    fn it_can_bound_motion() {
        let to = Transform::translate(&Vector::new_with(1.0, 1.0, 1.0)) *
            Transform::rotate_y(45.0);
        let anim_xform = AnimatedTransform::new(
            Transform::new(), 1.0, to.clone(), 2.0);
        let simple_box = BBox::new_with(Point::new_with(-1.0, -1.0, -1.0),
                                        Point::new_with(1.0, 1.0, 1.0));
        assert_eq!(anim_xform.motion_bounds(&simple_box, false),
                   BBox::new_with(Point::new_with(-1.0, -1.0, -1.0),
                                  Point::new_with(1.0 + 2f32.sqrt(),
                                                  2.0,
                                                  1.0 + 2f32.sqrt())));

        assert_eq!(anim_xform.motion_bounds(&simple_box, true),
                   // !KLUDGE! This x-value *looks* right but may not *be* right...
                   // Basically I have little intuitive sense for what happens to the
                   // box if you translate before you rotate...
                   BBox::new_with(Point::new_with(-1.6106478, -2.0,
                                                  -2.0*2f32.sqrt()),
                                  Point::new_with(2f32.sqrt(), 1.0, 1.0)));

        assert_eq!(AnimatedTransform::new(
            Transform::new(), 0.0, Transform::new(), 1.0).
                   motion_bounds(&simple_box, false), simple_box);

        assert_eq!(AnimatedTransform::new(
            Transform::new(), 0.0, to, 0.0).
                   motion_bounds(&simple_box, false), simple_box);
    }

    #[test]
    fn it_can_transform_points() {
        let from = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0));
        let to = Transform::translate(&Vector::new_with(-3.0, 0.0, -14.0));
        let mut xform = AnimatedTransform::new(from, 0.0, to, 1.0);
        assert_eq!(xform.xfpt(0.0, Point::new()), Point::new_with(1.0, 2.0, 3.0));
        assert_eq!(xform.xfpt(1.0, Point::new()), Point::new_with(-3.0, 0.0, -14.0));
        assert_eq!(xform.xfpt(0.5, Point::new()), Point::new_with(-1.0, 1.0, -5.5));

        assert_eq!(xform.tpt(0.0, &Point::new()), Point::new_with(1.0, 2.0, 3.0));
        assert_eq!(xform.tpt(1.0, &Point::new()), Point::new_with(-3.0, 0.0, -14.0));
        assert_eq!(xform.tpt(0.5, &Point::new()), Point::new_with(-1.0, 1.0, -5.5));

        let from2 = Transform::new();
        let to2 = Transform::translate(&Vector::new_with(1.0, 1.0, 1.0)) *
            Transform::scale(2.0, 1.5, 0.5) *
            Transform::rotate_x(45.0);
        xform = AnimatedTransform::new(from2, 0.0, to2, 10.0);
        let pt = Point::new_with(1.0, 1.0, 1.0);
        let expected = Point::new_with(3.0, 1.0, 1.0 + 0.5*2f32.sqrt());

        assert_eq!(xform.xfpt(0.0, pt.clone()), pt);
        assert_eq!(xform.xfpt(10.0, pt.clone()), expected);

        // !KLUDGE! These numbers look right -- but they might not *be* right....
        assert_eq!(xform.xfpt(5.0, pt.clone()),
                   Point::new_with(2.0, 0.90589696, 1.4799222));

        let xfpt = xform.xfpt(9.99999, pt.clone());
        assert!((xfpt.x - expected.x).abs() < 1e-5);
        assert!((xfpt.y - expected.y).abs() < 1e-5);
        assert!((xfpt.z - expected.z).abs() < 1e-5);
    }

    #[test]
    fn it_can_transform_vectors() {
        let from = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0));
        let to = Transform::translate(&Vector::new_with(-3.0, 0.0, -14.0));
        let mut xform = AnimatedTransform::new(from, 0.0, to, 1.0);

        // No matter what, just translated transforms shouldn't change
        // vectors...
        let random_vector = Vector::new_with(1.0, -10.3, 16.2);
        assert_eq!(xform.xfvec(0.0, random_vector.clone()), random_vector);
        assert_eq!(xform.xfvec(1.0, random_vector.clone()), random_vector);
        assert_eq!(xform.xfvec(0.5, random_vector.clone()), random_vector);

        assert_eq!(xform.tvec(0.0, &random_vector), random_vector);
        assert_eq!(xform.tvec(1.0, &random_vector), random_vector);
        assert_eq!(xform.tvec(0.5, &random_vector), random_vector);

        let from2 = Transform::new();
        let to2 = Transform::translate(&Vector::new_with(1.0, 1.0, 1.0)) *
            Transform::scale(2.0, 1.5, 0.5) *
            Transform::rotate_x(45.0);
        xform = AnimatedTransform::new(from2, 0.0, to2, 10.0);
        let v = Vector::new_with(1.0, 1.0, 1.0);
        let expected = Vector::new_with(2.0, 0.0, 0.5*2f32.sqrt());

        assert_eq!(xform.xfvec(0.0, v.clone()), v);
        assert_eq!(xform.xfvec(10.0, v.clone()), expected);

        // !KLUDGE! These numbers look right -- but they might not *be* right....
        assert_eq!(xform.xfvec(5.0, v.clone()),
                   Vector::new_with(1.5, 0.40589696, 0.9799222));

        let xfvec = xform.xfvec(9.99999, v.clone());
        assert!((xfvec.x - expected.x).abs() < 1e-5);
        assert!((xfvec.y - expected.y).abs() < 1e-5);
        assert!((xfvec.z - expected.z).abs() < 1e-5);
    }
}
