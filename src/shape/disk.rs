use bbox::BBox;
use geometry::point::Point;
use shape::shape::IsShape;
use shape::shape::Shape;
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
}

impl IsShape for Disk {
    fn get_shape<'a>(&'a self) -> &'a Shape { &self.shape }

    fn object_bound(&self) -> BBox {
        BBox::new_with(
            Point::new_with(-self.radius, -self.radius, self.height),
            Point::new_with(self.radius, self.radius, self.height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::f32::consts::PI;

    use bbox::BBox;
    use geometry::point::Point;
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
}
