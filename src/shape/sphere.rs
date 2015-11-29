use bbox::BBox;
use diff_geom::DifferentialGeometry;
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
}

impl IsShape for Sphere {
    fn get_shape<'a>(&'a self) -> &'a Shape { &self.shape }
    fn object_bound(&self) -> BBox {
        BBox::new_with(
            Point::new_with(-self.radius, -self.radius, self.z_min),
            Point::new_with(self.radius, self.radius, self.z_max))
    }

    fn intersect(&self, r: &Ray) -> Option<ShapeIntersection> {
        let phi : f32 = 0.0;
        let phit : Point = Point::new();

        // Transform ray to object space
        let ray = self.get_shape().world2object.t(r);

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
        // Test sphere intersection against clipping parameters
        // Find parametric representation of sphere hit
        // Initialize DifferentialGeometry from parametric information
        let dg : DifferentialGeometry = DifferentialGeometry::new();

        // update t_hit for quadric intersection
        let t_hit: f32 = 0.0;

        // compute ray_epsilon for quadric intersection
        let ray_epsilon : f32 = 0.0;

        Some(ShapeIntersection::new(t_hit, ray_epsilon, dg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::vector::Vector;
    use geometry::point::Point;
    use ray::Ray;
    use shape::shape::Shape;
    use shape::shape::IsShape;
    use shape::shape::ShapeIntersection;
    use transform::transform::Transform;

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

        assert!(s.intersect(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 1.5, 1.0), 0.0)).is_some());
        assert!(s.intersect(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 1.0, 1.0), 0.0)).is_some());
        assert_eq!(None, s.intersect(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 0.5, 1.0), 0.0)));
        assert!(s.intersect(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(0.0, 2.0, 1.0), 0.0)).is_some());
        assert!(s.intersect(
            &Ray::new_with(Point::new_with(0.0, 0.0, 0.0),
                           Vector::new_with(1.0, 2.0, 0.0), 0.0)).is_some());

        // A non-full sphere should also be able to be intersected...
        let xf2 = Transform::translate(&Vector::new_with(0.0, -3.0, 0.0))
            * Transform::scale(2.0, 2.0, 2.0);
        let xf2_inv = xf2.inverse();
        let s2 = Sphere::new(xf2, xf2_inv, false, 0.75, 0.5, 0.75, 180.0);
        assert!(s2.can_intersect());
        assert!(s2.intersect(
            &Ray::new_with(Point::new(),
                           Vector::new_with(0.0, -1.0, 0.0), 0.0)).is_some());
        assert_eq!(None, s2.intersect(
            &Ray::new_with(Point::new_with(0.0, -4.0, 10.0),
                           Vector::new_with(0.0, 0.0, -1.0), 0.0)));
    }
}
