use bbox::BBox;
use geometry::point::Point;
use shape::shape::IsShape;
use shape::shape::Shape;
use shape::shape::ShapeIntersection;
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
}

impl IsShape for Cylinder {
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
    use bbox::BBox;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use shape::shape::IsShape;
    use shape::shape::Shape;
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
}
