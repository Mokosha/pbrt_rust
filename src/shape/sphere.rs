use bbox::BBox;
use geometry::point::Point;
use geometry::vector::Dot;
use geometry::vector::Vector;
use ray::Ray;
use shape::shape::Shape;
use shape::shape::ShapeIntersection;
use shape::shape::IsShape;
use transform::transform::Transform;
use transform::transform::ApplyTransform;
use utils::Degrees;
use utils::Clamp;

use shape::helpers::compute_dg;

#[derive(Debug, PartialEq, Clone)]
pub struct Sphere {
    shape: Shape,
    radius: f32,
    phi_max: f32,
    z_min: f32,
    z_max: f32,
    theta_min: f32,
    theta_max: f32
}

impl Sphere {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool,
               rad: f32, z0: f32, z1: f32, pm: f32) -> Sphere {
        debug_assert!(rad > 0f32);
        let zmin = z0.min(z1).clamp(-rad, rad);
        let zmax = z0.max(z1).clamp(-rad, rad);
        let thetamin = (zmin / rad).acos();
        let thetamax = (zmax / rad).acos();
        Sphere {
            shape: Shape::new(o2w, w2o, ro),
            radius: rad,
            z_min: zmin,
            z_max: zmax,
            theta_min: thetamin,
            theta_max: thetamax,
            phi_max: pm.clamp(0.0, 360.0).as_radians()
        }
    }

    fn get_intersection_point(&self, ray: &Ray) -> Option<(f32, f32)> {
        // Compute quadratic sphere coefficients
        let a = ray.d.length_squared();
        let b = 2.0 * ray.d.dot(&Vector::from(ray.o.clone()));
        let c = Vector::from(ray.o.clone()).length_squared() -
            self.radius * self.radius;

        // Solve quadratic equation for t values
        let (t0, t1) = {
            match ::utils::quadratic(a, b, c) {
                None => return None,
                Some((x, y)) => (x, y)
            }
        };

        // Compute intersection distance along ray
        if t0 > ray.maxt || t1 < ray.mint {
            return None
        }

        let mut t_hit = t0;
        if t0 < ray.mint {
            t_hit = t1;
            if t_hit > ray.maxt {
                return None;
            }
        }

        // Compute sphere hit position and phi
        let get_hit = |t: f32| {
            let mut hit = ray.point_at(t);
            if hit.x == 0.0 && hit.y == 0.0 {
                hit.x = 1e-5 * self.radius;
            }

            let mut angle = hit.y.atan2(hit.x);
            if angle < 0.0 {
                angle += 2.0 * ::std::f32::consts::PI;
            }
            (hit, angle)
        };

        let hit_is_invalid = |hit: &(Point, f32)| {
            (hit.0.z > -self.radius && hit.0.z < self.z_min) ||
                (hit.0.z <  self.radius && hit.0.z > self.z_max) ||
                (hit.1 > self.phi_max)
        };

        // Test sphere intersection against clipping parameters
        let mut test = get_hit(t_hit);
        if hit_is_invalid(&test) {
            if t_hit == t1 { return None; }
            if t1 > ray.maxt { return None; }
            t_hit = t1;
            test = get_hit(t_hit);
            if hit_is_invalid(&test) {
                return None;
            }
        }

        Some((t_hit, test.1))
    }
}

impl<'a> IsShape<'a> for Sphere {
    fn get_shape(&'a self) -> &'a Shape { &self.shape }
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

        // Find parametric representation of sphere hit
        let u = phi / self.phi_max;
        let theta = (p_hit.z / self.radius).clamp(-1.0, 1.0).acos();
        let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);
        
        // Compute dp/du and dp/dv
        let zradius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
        let inv_zradius = 1.0 / zradius;
        let cos_phi = p_hit.x * inv_zradius;
        let sin_phi = p_hit.y * inv_zradius;

        let dpdu = Vector::new_with(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = (self.theta_max - self.theta_min) *
            Vector::new_with(p_hit.z * cos_phi,
                             p_hit.z * sin_phi,
                             -self.radius * theta.sin());

        // Compute dn/du and dn/dv
        let d2pduu = -self.phi_max * self.phi_max * Vector::new_with(p_hit.x, p_hit.y, 0.0);
        let d2pduv = (self.theta_max - self.theta_min) * p_hit.z * self.phi_max *
            Vector::new_with(-sin_phi, cos_phi, 0.0);
        let d2pdvv =
            -(self.theta_max - self.theta_min) *
            (self.theta_max - self.theta_min) *
            Vector::from(p_hit.clone());

        let dg = compute_dg(self.get_shape(), u, v, p_hit,
                            dpdu, dpdv, d2pduu, d2pduv, d2pdvv);
        Some(ShapeIntersection::new(t_hit, t_hit * 5e-4, dg))
    }

    fn area(&self) -> f32 {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::normal::Normal;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use ray::Ray;
    use shape::shape::Shape;
    use shape::shape::IsShape;
    use transform::transform::Transform;

    use std::f32::consts::PI;

    #[test]
    fn it_can_be_created() {
        assert_eq!(Sphere::new(Transform::new(), Transform::new(),
                               false, 1.0, -1.0, 1.0, 360.0),
                   Sphere {
                       shape: Shape::new(Transform::new(), Transform::new(), false),
                       radius: 1.0,
                       z_min: -1.0,
                       z_max: 1.0,
                       // This is PI but due to floating point precision errors,
                       // we want to compare against whatever approximated version
                       // of PI we actually produce...
                       theta_min: (-1f32).acos(),
                       theta_max: 0.0,
                       phi_max: ::std::f32::consts::PI * 2.0
                   });
    }

    #[test]
    fn it_can_be_intersected() {
        let xf = Transform::translate(&Vector::new_with(1.0, 2.0, 1.0));
        let xf_inv = xf.inverse();
        let s = Sphere::new(xf, xf_inv, false, 1.0, -1.0, 1.0, 360.0);

        // A full sphere should be able to be intersected at will...
        assert!(s.can_intersect());

        // !FIXME! We need to actually investigate that our ray hits
        // where we think it does rather than it just hits at all...

        assert!(s.intersect_p(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 1.5, 1.0), 0.0)));
        assert!(s.intersect_p(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 1.0, 1.0), 0.0)));
        assert!(!s.intersect_p(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 0.5, 1.0), 0.0)));
        assert!(s.intersect_p(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(0.0, 2.0, 1.0), 0.0)));
        assert!(s.intersect_p(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 2.0, 0.0), 0.0)));

        // A non-full sphere should also be able to be intersected...
        let xf2 = Transform::translate(&Vector::new_with(0.0, -3.0, 0.0))
            * Transform::scale(2.0, 2.0, 2.0);
        let xf2_inv = xf2.inverse();
        let s2 = Sphere::new(xf2.clone(), xf2_inv.clone(), false, 0.75, -0.5, 0.75, 180.0);
        let straight_down = Ray::new_with(Point::new(), Vector::new_with(0.0, -1.0, 0.0), 0.0);
        assert!(s2.can_intersect());

        // Check against z-bounds
        assert!(!Sphere::new(xf2.clone(), xf2_inv.clone(), false, 0.75, -0.75, -0.5, 180.0)
                .intersect_p(&straight_down));
        assert!(!Sphere::new(xf2.clone(), xf2_inv.clone(), false, 0.75, 0.5, 0.75, 180.0)
                .intersect_p(&straight_down));

        // If we do go straight down, it should be fine...
        assert!(s2.intersect(&straight_down).is_some());

        // If we start in the middle of the sphere, it should not
        assert!(!s2.intersect_p(
            &Ray::new_with(Point::new_with(0.0, -3.0, 0.0),
                           Vector::new_with(0.0, -1.0, 0.0), 0.0)));

        // If we go straight up, though we should hit it...
        assert!(s2.intersect_p(
            &Ray::new_with(Point::new_with(0.0, -3.0, 0.0),
                           Vector::new_with(0.0, 1.0, 0.0), 0.0)));

        // If we graze the sphere in some awkward way, no go either.
        assert!(!s2.intersect_p(
            &Ray::new_with(Point::new_with(0.0, -4.0, 10.0),
                           Vector::new_with(0.0, 0.0, -1.0), 0.0)));
    }

    #[test]
    fn it_has_intersection_information() {
        let xf = Transform::translate(&Vector::new_with(0.0, -1.0, 0.0));
        let xf_inv = xf.inverse();
        let s = Sphere::new(xf, xf_inv, false, 0.5, -0.5, 0.5, 360.0);

        let r = Ray::new_with(
            Point::new(), Vector::new_with(0.0, -1.0, 0.0), 0.0);
        let shape_int = s.intersect(&r).unwrap();

        assert_eq!(shape_int.t_hit, 0.5);
        assert_eq!(shape_int.ray_epsilon, 0.5 * 5e-4);
        assert_eq!(shape_int.dg.p, Point::new_with(0.0, -0.5, 0.0));
        assert_eq!(shape_int.dg.shape.unwrap(), s.get_shape());

        let expected_normal = Normal::new_with(0.0, 1.0, 0.0);
        assert_eq!(shape_int.dg.nn.x, expected_normal.x);
        assert!((shape_int.dg.nn.y - expected_normal.y).abs() < 1e-6);
        assert_eq!(shape_int.dg.nn.z, expected_normal.z);
        assert_eq!(shape_int.dg.u, 0.25); // A quarter of a full revolution (phi)
        assert!((shape_int.dg.v - 0.5).abs() < 1e-6);  // A half of a half revolution (theta)
        assert!((shape_int.dg.dpdu.x + PI).abs() < 1e-6);
        assert_eq!(shape_int.dg.dpdu.y, 0.0);
        assert_eq!(shape_int.dg.dpdu.z, 0.0);
        assert_eq!(shape_int.dg.dpdv.x, 0.0);
        assert_eq!(shape_int.dg.dpdv.y, 0.0);
        assert!((shape_int.dg.dpdv.z - (PI / 2.0)).abs() < 1e-6);
    }

    #[test]
    fn it_has_a_surface_area() {
        // Sphere surface area is 4 PI r^2...
        assert_eq!(Sphere::new(
            Transform::new(), Transform::new(), false,
            1.0, -1.0, 1.0, 360.0).area(), 4.0 * PI);

        assert_eq!(Sphere::new(
            Transform::new(), Transform::new(), false,
            0.5, -1.0, 1.0, 360.0).area(), PI);

        // Half spheres should be half as big...
        assert_eq!(Sphere::new(
            Transform::new(), Transform::new(), false,
            1.0, -1.0, 1.0, 180.0).area(), 2.0 * PI);

        assert_eq!(Sphere::new(
            Transform::new(), Transform::new(), false,
            0.5, 0.0, 1.0, 360.0).area(), 0.5 * PI);

        assert_eq!(Sphere::new(
            Transform::new(), Transform::new(), false,
            0.5, -1.0, 0.0, 360.0).area(), 0.5 * PI);

        // If we transform the sphere (i.e. scale it) it
        // logically changes its surface area, but not
        // functionally...
        let xf = Transform::scale(0.5, 2.0, 1.0);
        assert_eq!(Sphere::new(
            xf.clone(), xf.inverse(), false,
            1.0, -1.0, 1.0, 360.0).area(), 4.0 * PI);

        // Certainly other transforms shouldn't do anything
        let xf2 = Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
            * Transform::rotate_x(30.0) * xf;
        assert_eq!(Sphere::new(
            xf2.clone(), xf2.inverse(), false,
            1.0, -1.0, 1.0, 360.0).area(), 4.0 * PI);
    }
}
