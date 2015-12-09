use bbox::BBox;
use geometry::point::Point;
use ray::Ray;
use shape::shape::IsShape;
use shape::shape::Shape;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Clamp;
use utils::Degrees;

#[derive(Debug, PartialEq, Clone)]
pub struct Disk {
    shape: Shape,
    height: f32,
    radius: f32,
    inner_radius: f32,
    phi_max: f32
}

impl Disk {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool,
               ht: f32, r: f32, ri: f32, t_max: f32) -> Disk {
        Disk {
            shape: Shape::new(o2w, w2o, ro),
            height: ht,
            radius: r,
            inner_radius: ri,
            phi_max: t_max.clamp(0.0, 360.0).as_radians()
        }
    }

    fn get_intersection_point(&self, r: &Ray) -> Option<(f32, f32)> {
        // Assume ray is transformed into object space.

        // Compute plane intersection for disk
        if r.d.z.abs() < 1e-6 {
            // Ray parallel to the plane won't intersect ever.
            return None;
        }

        let t_hit = (self.height - r.o.z) / r.d.z;
        if t_hit < r.mint || t_hit > r.maxt {
            return None;
        }

        // See if hit point is inside disk radii
        let p_hit = r.point_at(t_hit);
        let dist2 = p_hit.x * p_hit.x + p_hit.y * p_hit.y;
        if (dist2 > (self.radius * self.radius) ||
            dist2 < (self.inner_radius * self.inner_radius)) {
            return None;
        }

        // See if hit point is inside phi_max
        let phi = {
            let a = p_hit.y.atan2(p_hit.x);
            if a < 0.0 {
                a + 2.0 * ::std::f32::consts::PI
            } else {
                a
            }
        };

        if phi > self.phi_max { None } else { Some((t_hit, phi)) }
    }
}

impl IsShape for Disk {
    fn get_shape<'a>(&'a self) -> &'a Shape { &self.shape }

    fn object_bound(&self) -> BBox {
        BBox::new_with(
            Point::new_with(-self.radius, -self.radius, self.height),
            Point::new_with(self.radius, self.radius, self.height))
    }

    fn intersect_p(&self, r: &Ray) -> bool {
        // Transform ray to object space
        let ray = self.get_shape().world2object.t(r);
        self.get_intersection_point(&ray).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::f32::consts::PI;

    use bbox::BBox;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use ray::Ray;
    use shape::shape::IsShape;
    use shape::shape::Shape;
    use transform::transform::Transform;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Disk::new(Transform::new(), Transform::new(), false,
                             0.0, 1.0, 0.5, 360.0),
                   Disk {
                       shape: Shape::new(Transform::new(), Transform::new(), false),
                       height: 0.0,
                       radius: 1.0,
                       inner_radius: 0.5,
                       phi_max: 2.0 * PI
                   });

        let xf = Transform::scale(1.0, 2.0, 3.0);
        assert_eq!(Disk::new(xf.clone(), xf.inverse(), false, 2.0, 0.0, 1.0, 90.0),
                   Disk {
                       shape: Shape::new(xf.clone(), xf.inverse(), false),
                       height: 2.0,
                       radius: 0.0,
                       inner_radius: 1.0,
                       phi_max: 0.5 * PI
                   });
    }

    #[test]
    fn it_has_object_bounds() {
        assert_eq!(Disk::new(Transform::new(), Transform::new(), false,
                             0.0, 1.0, 0.5, 360.0).object_bound(),
                   BBox::new_with(
                       Point::new_with(-1.0, -1.0, 0.0),
                       Point::new_with(1.0, 1.0, 0.0)));

        let xf = Transform::scale(1.0, 2.0, 3.0);
        assert_eq!(Disk::new(xf.clone(), xf.inverse(), false, 2.0, 0.0, 1.0, 90.0)
                   .object_bound(),
                   BBox::new_with(
                       Point::new_with(0.0, 0.0, 2.0),
                       Point::new_with(0.0, 0.0, 2.0)));
    }

    #[test]
    fn it_can_be_intersected() {
        // First, can we intersect a simple disk?
        assert!(Disk::new(Transform::new(), Transform::new(), false,
                          0.0, 1.0, 0.0, 360.0).intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, 0.0, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        assert!(!Disk::new(Transform::new(), Transform::new(), false,
                          0.0, 1.0, 0.0, 360.0).intersect_p(
            &Ray::new_with(
                Point::new_with(2.0, 2.0, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        // See if we can shoot right through the hole...
        assert!(!Disk::new(Transform::new(), Transform::new(), false,
                          0.0, 1.0, 0.5, 360.0).intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, 0.0, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        // See if we miss starting behind the disk...
        assert!(!Disk::new(Transform::new(), Transform::new(), false,
                           2.0, 1.0, 0.0, 360.0).intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, 0.0, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        // Hit the top half of a half-pipe
        let half_pipe = Disk::new(Transform::new(), Transform::new(), false,
                                  0.0, 0.75, 0.25, 180.0);
        assert!(half_pipe.intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, 0.5, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        // But miss the bottom half...
        assert!(!half_pipe.intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, -0.5, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        // If we rotate the half pipe, it should be the opposite
        let xf = Transform::rotate_z(180.0);
        let half_pipe2 = Disk::new(xf.clone(), xf.inverse(), false,
                                   0.0, 0.75, 0.25, 180.0);
        assert!(!half_pipe2.intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, 0.5, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));
        assert!(half_pipe2.intersect_p(
            &Ray::new_with(
                Point::new_with(0.0, -0.5, 1.0),
                Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        assert!(!half_pipe2.intersect_p(
            &Ray::new_with(
                Point::new_with(1.0, 10.5, 140.0),
                Vector::new_with(-1.0, -10.5, -140.0), 0.0)));
    }
}
