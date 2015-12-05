use std::f32::consts::PI;

use bbox::BBox;
use geometry::point::Point;
use ray::Ray;
use shape::shape::IsShape;
use shape::shape::Shape;
use shape::shape::ShapeIntersection;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Clamp;
use utils::Degrees;

#[derive(Debug, PartialEq, Clone)]
pub struct Cylinder {
    shape: Shape,
    radius: f32,
    z_min: f32,
    z_max: f32,
    phi_max: f32
}

impl Cylinder {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool,
               rad: f32, z0: f32, z1: f32, pm: f32) -> Cylinder {
        Cylinder {
            shape: Shape::new(o2w, w2o, ro),
            radius: rad,
            z_min: z0.min(z1),
            z_max: z0.max(z1),
            phi_max: pm.clamp(0.0, 360.0).as_radians()
        }
    }

    fn get_intersection_point(&self, r: &Ray) -> Option<(f32, f32)> {
        // Compute quadratic cylinder coefficients
        let a = r.d.x * r.d.x + r.d.y * r.d.y;
        let b = 2.0 * (r.d.x * r.o.x + r.d.y * r.o.y);
        let c = r.o.x * r.o.x + r.o.y * r.o.y - self.radius * self.radius;

        // Solve quadratic equation for t values
        let (t0, t1) = {
            match ::utils::quadratic(a, b, c) {
                None => return None,
                Some((x, y)) => (x, y)
            }
        };

        // Compute intersection distance along ray
        if t0 > r.maxt || t1 < r.mint {
            return None
        }

        let mut t_hit = t0;
        if t0 < r.mint {
            t_hit = t1;
            if t_hit > r.maxt {
                return None;
            }
        }

        // Compute cylinder hit point and Phi
        let get_hit = |t: f32| {
            let mut hit = r.point_at(t);
            if hit.x == 0.0 && hit.y == 0.0 {
                hit.x = 1e-5 * self.radius;
            }

            let mut angle = hit.y.atan2(hit.x);
            if angle < 0.0 {
                angle = angle + 2.0 * PI;
            }
            (hit, angle)
        };

        let invalid_hit = |hit: &(Point, f32)| {
            hit.0.z < self.z_min || hit.0.z > self.z_max || hit.1 > self.phi_max
        };

        // Test cylinder intersection against clipping parameters
        let mut p_hit = get_hit(t_hit);
        if invalid_hit(&p_hit) {
            if t_hit == t1 { return None; }
            if t1 > r.maxt { return None; }
            t_hit = t1;
            p_hit = get_hit(t_hit);
            if invalid_hit(&p_hit) { return None; }
        }

        return Some((t_hit, p_hit.1))
    }
}

impl IsShape for Cylinder {
    fn get_shape<'a>(&'a self) -> &'a Shape { &self.shape }
    fn object_bound(&self) -> BBox {
        BBox::new_with(
            Point::new_with(-self.radius, -self.radius, self.z_min),
            Point::new_with(self.radius, self.radius, self.z_max))
    }

    fn intersect_p(&self, r: &Ray) -> bool {
        // Transform ray to object space
        let ray = self.get_shape().world2object.t(r);
        self.get_intersection_point(&ray).is_some()
    }

    fn intersect(&self, r: &Ray) -> Option<ShapeIntersection> {
        // Transform ray to object space
        let ray = self.get_shape().world2object.t(r);

        let (t_hit, phi) = {
            let hit = self.get_intersection_point(&ray);
            if hit.is_some() { hit.unwrap() } else { return None; }
        };

        let p_hit = ray.point_at(t_hit);

        // Find parametric representation of cylinder hit

        /*
        // Initialize DifferentialGeometry from parametric information
        let o2w = &(self.get_shape().object2world);

        let dg : DifferentialGeometry = DifferentialGeometry::new_with(
            o2w.xf(p_hit), o2w.xf(dpdu), o2w.xf(dpdv), o2w.xf(dndu),
            o2w.xf(dndv), u, v, Some(self.get_shape()));

        Some(ShapeIntersection::new(t_hit, t_hit * 5e-4, dg))
         */
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbox::BBox;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use ray::Ray;
    use shape::shape::IsShape;
    use shape::shape::Shape;
    use transform::transform::ApplyTransform;
    use transform::transform::Transform;
    use utils::Degrees;

    #[test]
    fn it_can_be_created() {
        let xf = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0));
        assert_eq!(Cylinder::new(xf.clone(), xf.inverse(), false,
                                 3.2, 14.0, -3.0, 16.0),
                   Cylinder {
                       shape: Shape::new(xf.clone(), xf.inverse(), false),
                       radius: 3.2,
                       z_min: -3.0,
                       z_max: 14.0,
                       phi_max: 16f32.as_radians()
                   });
    }

    #[test]
    fn it_has_bounds() {
        assert_eq!(Cylinder::new(Transform::new(), Transform::new(), false,
                                 0.5, -2.0, -1.0, 360.0).object_bound(),
                   BBox::new_with(
                       Point::new_with(-0.5, -0.5, -2.0),
                       Point::new_with(0.5, 0.5, -1.0)));

        // It ignores the transform and phi_max when computing the bounds
        let xf = Transform::scale(2.0, 3.0, 0.2);
        assert_eq!(Cylinder::new(xf.clone(), xf.inverse(), false,
                                 0.5, -2.0, -1.0, 360.0).object_bound(),
                   BBox::new_with(
                       Point::new_with(-0.5, -0.5, -2.0),
                       Point::new_with(0.5, 0.5, -1.0)));

        assert_eq!(Cylinder::new(Transform::new(), Transform::new(), false,
                                 0.5, -2.0, -1.0, 180.0).object_bound(),
                   BBox::new_with(
                       Point::new_with(-0.5, -0.5, -2.0),
                       Point::new_with(0.5, 0.5, -1.0)));
    }

    #[test]
    fn it_can_be_intersected() {
        let simple = Cylinder::new(Transform::new(), Transform::new(), false,
                                   0.5, -1.0, 1.0, 360.0);
        assert!(simple.intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, 1.0),
                           Vector::new_with(-1.0, -1.0, -1.0), 0.0)));

        assert!(!simple.intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, 1.0),
                           Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        // !FIXME! Add a couple more stringent tests...
    }
}
