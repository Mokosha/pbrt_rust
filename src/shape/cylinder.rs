use std::f32::consts::PI;

use bbox::BBox;
use bbox::HasBounds;
use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use shape::shape::ShapeBase;
use shape::shape::ShapeIntersection;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Clamp;
use utils::Degrees;

use shape::helpers::compute_dg;

#[derive(Debug, PartialEq, Clone)]
pub struct Cylinder {
    base: ShapeBase,
    radius: f32,
    z_min: f32,
    z_max: f32,
    phi_max: f32
}

impl Cylinder {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool,
               rad: f32, z0: f32, z1: f32, pm: f32) -> Cylinder {
        Cylinder {
            base: ShapeBase::new(o2w, w2o, ro),
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

    pub fn base<'a>(&'a self) -> &'a ShapeBase { &self.base }

    pub fn object_bound(&self) -> BBox {
        BBox::new_with(
            Point::new_with(-self.radius, -self.radius, self.z_min),
            Point::new_with(self.radius, self.radius, self.z_max))
    }

    pub fn area(&self) -> f32 {
        // Unroll the rectangle
        (self.z_max - self.z_min) * self.phi_max * self.radius
    }
}

impl HasBounds for Cylinder {
    fn world_bound(&self) -> BBox {
        self.base().object2world.xf(self.object_bound())
    }
}

impl<'a> Intersectable<'a, ShapeIntersection<'a>> for Cylinder {
    fn intersect_p(&self, r: &Ray) -> bool {
        // Transform ray to object space
        let ray = self.base().world2object.t(r);
        self.get_intersection_point(&ray).is_some()
    }

    fn intersect(&self, r: &Ray) -> Option<ShapeIntersection> {
        // Transform ray to object space
        let ray = self.base().world2object.t(r);

        let (t_hit, phi) = {
            let hit = self.get_intersection_point(&ray);
            if hit.is_some() { hit.unwrap() } else { return None; }
        };

        let p_hit = ray.point_at(t_hit);

        // Find parametric representation of cylinder hit
        let u = phi / self.phi_max;
        let v = (p_hit.z - self.z_min) / (self.z_max - self.z_min);

        // Compute cylinder dpdu and dpdv
        let dpdu = self.phi_max * Vector::new_with(-p_hit.y, p_hit.x, 0.0);
        let dpdv = Vector::new_with(0.0, 0.0, self.z_max - self.z_min);

        // Compute cylinder dndu and dndv
        let d2pduu = -self.phi_max * self.phi_max *
            Vector::new_with(p_hit.x, p_hit.y, 0.0);
        let d2pduv = Vector::new();
        let d2pdvv = Vector::new();

        // Initialize DifferentialGeometry from parametric information
        let dg = compute_dg(self.base(), u, v, p_hit,
                            dpdu, dpdv, d2pduu, d2pduv, d2pdvv);

        Some(ShapeIntersection::new(t_hit, t_hit * 5e-4, dg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::f32::consts::PI;

    use bbox::BBox;
    use geometry::point::Point;
    use geometry::normal::Normalize;
    use geometry::vector::Vector;
    use intersection::Intersectable;
    use ray::Ray;
    use shape::shape::ShapeBase;
    use transform::transform::Transform;
    use utils::Degrees;

    #[test]
    fn it_can_be_created() {
        let xf = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0));
        assert_eq!(Cylinder::new(xf.clone(), xf.inverse(), false,
                                 3.2, 14.0, -3.0, 16.0),
                   Cylinder {
                       base: ShapeBase::new(xf.clone(), xf.inverse(), false),
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

        assert!(!simple.intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, 2.0),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));

        assert!(!simple.intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, 2.0),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));

        assert!(!simple.intersect_p(
            &Ray::new_with(Point::new_with(-1.0, 1.0, -2.0),
                           Vector::new_with(1.0, -1.0, 0.0), 0.0)));

        // Shooting down the middle of a cylinder shouldn't work either...
        assert!(!simple.intersect_p(
            &Ray::new_with(Point::new_with(0.1, 0.1, -2.0),
                           Vector::new_with(0.0, 0.0, 1.0), 0.0)));

        // Hitting the cylinder from the inside should work...
        assert!(simple.intersect_p(
            &Ray::new_with(Point::new_with(0.1, 0.1, -2.0),
                           Vector::new_with(-0.4, -0.4, 1.5), 0.0)));

        // Test partial cylinder...
        let xf = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0));
        let partial = Cylinder::new(xf.clone(), xf.inverse(), false,
                                    1.0, -3.0, 0.0, 90.0);

        assert!(partial.intersect_p(
            &Ray::new_with(Point::new_with(2.0, 4.0, 1.5),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));

        assert!(!partial.intersect_p(
            &Ray::new_with(Point::new_with(1.0, 2.0, 1.5),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));

        assert!(partial.intersect_p(
            &Ray::new_with(Point::new_with(1.0, 2.0, 1.5),
                           Vector::new_with(1.0, 1.0, 0.0), 0.0)));

        // Finally, one that just barely misses...
        assert!(!partial.intersect_p(
            &Ray::new_with(Point::new_with(2.5, 2.5, 10.0),
                           Vector::new_with(-1.0, -1.0, -10.0), 0.0)));

        // Can we intersect one with no radius?
        assert!(Cylinder::new(Transform::new(), Transform::new(), false,
                              0.0, -1.0, 1.0, 360.0).intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, -0.5),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));            

        // Even if it has a really tiny phi_max?
        assert!(Cylinder::new(Transform::new(), Transform::new(), false,
                              0.0, -1.0, 1.0, 0.1).intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, -0.5),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));            

        // What if it has no phi_max? (This makes sense, since all intersection
        // points have phi = 0,
        assert!(Cylinder::new(Transform::new(), Transform::new(), false,
                              0.0, -1.0, 1.0, 0.0).intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, -0.5),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));            

        // Negative phi_max gets clamped?
        assert!(Cylinder::new(Transform::new(), Transform::new(), false,
                              0.0, -1.0, 1.0, -1.0).intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, -0.5),
                           Vector::new_with(-1.0, -1.0, 0.0), 0.0)));            

        // It should work even if we have a cylinder effectively defined
        // as the origin...
        assert!(Cylinder::new(Transform::new(), Transform::new(), false,
                              0.0, 0.0, 0.0, 360.0).intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, 1.0),
                           Vector::new_with(-1.0, -1.0, -1.0), 0.0)));            

        // But not if it's defined as some other point along the z-axis
        assert!(!Cylinder::new(Transform::new(), Transform::new(), false,
                               0.0, -1.0, -1.0, 360.0).intersect_p(
            &Ray::new_with(Point::new_with(1.0, 1.0, 1.0),
                           Vector::new_with(-1.0, -1.0, -1.0), 0.0)));            
    }

    #[test]
    fn it_has_intersection_information() {
        // Just like the sphere -- this looks right? idk...
        let xf = Transform::rotate_y(90.0);
        let c = Cylinder::new(xf.clone(), xf.inverse(), false,
                              1.0, -1.0, 1.0, 180.0);

        let r = Ray::new_with(Point::new_with(0.0, 1.0, 1.0),
                              Vector::new_with(0.0, -1.0, -1.0).normalize(), 0.0);
        let shape_int = c.intersect(&r).unwrap();

        assert!((shape_int.t_hit - (2f32.sqrt() - 1.0)).abs() < 1e-6);
        assert!((shape_int.ray_epsilon - ((2f32.sqrt() - 1.0) * 5e-4)).abs() < 1e-6);

        let sqrt2_2 = 2f32.sqrt() * 0.5;
        assert!((shape_int.dg.p -
                 Point::new_with(0.0, sqrt2_2, sqrt2_2)).length_squared() < 1e-6);

        assert_eq!(shape_int.dg.shape.unwrap(), c.base());
        assert!((Vector::from(shape_int.dg.nn) -
                 Vector::new_with(0.0, 1.0, 1.0).normalize()).length_squared() < 1e-6);

        assert_eq!(shape_int.dg.u, 0.75);
        assert!((shape_int.dg.v - 0.5).abs() < 1e-6);

        let expected_dpdu = Vector::new_with(0.0, -PI * sqrt2_2, PI * sqrt2_2);
        assert!((shape_int.dg.dpdu - expected_dpdu).length_squared() < 1e-6);

        let expected_dpdv = Vector::new_with(2.0, 0.0, 0.0);
        assert!((shape_int.dg.dpdv - expected_dpdv).length_squared() < 1e-6);
    }

    #[test]
    fn it_has_a_surface_area() {
        assert_eq!(Cylinder::new(Transform::new(), Transform::new(), false,
                                 1.0 / PI, 0.0, 1.0, 360.0).area(), 2.0);
        assert_eq!(Cylinder::new(Transform::new(), Transform::new(), false,
                                 1.0 / PI, 0.0, 1.0, 180.0).area(), 1.0);

        // It doesn't handle transforms...
        let xf = Transform::scale(1.0, 2.0, 0.5);
        assert_eq!(Cylinder::new(xf.clone(), xf.inverse(), false,
                                 1.0 / PI, 0.0, 1.0, 360.0).area(), 2.0);

        // It handles zero size cylinders...
        assert_eq!(Cylinder::new(xf.clone(), xf.inverse(), false,
                                 0.0, 0.0, 1.0, 360.0).area(), 0.0);
        assert_eq!(Cylinder::new(xf.clone(), xf.inverse(), false,
                                 1.0, 0.0, 0.0, 360.0).area(), 0.0);
    }
}
