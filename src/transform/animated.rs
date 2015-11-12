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

pub struct AnimatedTransform {
    start_time: f32,
    end_time: f32,
    start_transform: Transform,
    end_transform: Transform,
    actually_animated: bool,
    t: [Vector; 2],
    r: [Quaternion; 2],
    s: [Matrix4x4; 2]
}

impl AnimatedTransform {
    pub fn new(transform1: &Transform, time1: f32,
               transform2: &Transform, time2: f32) -> AnimatedTransform {
        let (t1, r1, s1) = AnimatedTransform::decompose(transform1);
        let (t2, r2, s2) = AnimatedTransform::decompose(transform2);
        AnimatedTransform {
            start_time: time1,
            end_time: time2,
            start_transform: transform1.clone(),
            end_transform: transform2.clone(),
            actually_animated: transform1 != transform2,
            t: [t1, t2],
            r: [r1, r2],
            s: [s1, s2]
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
        let trans = self.t[0].lerp(&self.t[1], dt);

        // Interpolate rotation at dt
        let rotate = self.r[0].lerp(&self.r[1], dt);

        // Interpolate scale at dt
        let scale = self.s[0].lerp(&self.s[1], dt);

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
            let r_next = {
                let r_it = r.clone().invert().transpose();
                0.5 * (&r + r_it)
            };

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
                                         ((i as f32) / (num_steps as f32)));
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
