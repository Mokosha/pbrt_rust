use bbox::BBox;
use geometry::point::Point;
use shape::shape::Shape;
use shape::shape::IsShape;
use transform::transform::Transform;
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
        println!("{:?}", (thetamin, zmin / rad, (-1f32).acos()));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use shape::shape::Shape;
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
}
