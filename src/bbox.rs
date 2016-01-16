use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use std::f32;
use ray::Ray;
use utils::Lerp;

pub trait Union<T = Self> : Sized {
    fn union(&self, &T) -> Self;
    fn unioned_with_ref(self, v: &T) -> Self {
        self.union(v)
    }
    fn unioned_with(self, v: T) -> Self {
        self.union(&v)
    }
}

pub trait HasBounds {
    fn world_bound(&self) -> BBox;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    pub p_min: Point,
    pub p_max: Point
}

impl BBox {
    pub fn new() -> BBox {
        BBox {
            p_min: Point::new_with(f32::MAX, f32::MAX, f32::MAX),
            p_max: Point::new_with(-f32::MAX, -f32::MAX, -f32::MAX),
        }
    }

    pub fn new_with(pmin: Point, pmax: Point) -> BBox {
        BBox {
            p_min: pmin,
            p_max: pmax
        }
    }

    pub fn empty(&self) -> bool {
        let d = &self.p_max - &self.p_min;
        d.x <= 0f32 || d.y <= 0f32 || d.z <= 0f32
    }

    pub fn overlaps(&self, b: &BBox) -> bool {
        let x = self.p_max.x >= b.p_min.x && self.p_min.x <= b.p_max.x;
        let y = self.p_max.y >= b.p_min.y && self.p_min.y <= b.p_max.y;
        let z = self.p_max.z >= b.p_min.z && self.p_min.z <= b.p_max.z;
        x && y && z
    }

    pub fn inside(&self, p: &Point) -> bool {
        p.x >= self.p_min.x && p.x <= self.p_max.x &&
            p.y >= self.p_min.y && p.y <= self.p_max.y &&
            p.z >= self.p_min.z && p.z <= self.p_max.z
    }

    pub fn expand(&mut self, delta: f32) {
        self.p_min = &(self.p_min) - Vector::new_with(delta, delta, delta);
        self.p_max = &(self.p_max) + Vector::new_with(delta, delta, delta);
    }

    pub fn surface_area(&self) -> f32 {
        let dx = (self.p_max.x - self.p_min.x).max(0f32);
        let dy = (self.p_max.y - self.p_min.y).max(0f32);
        let dz = (self.p_max.z - self.p_min.z).max(0f32);
        2f32 * (dx * dy + dx * dz + dy * dz)
    }

    pub fn volume(&self) -> f32 {
        let dx = (self.p_max.x - self.p_min.x).max(0f32);
        let dy = (self.p_max.y - self.p_min.y).max(0f32);
        let dz = (self.p_max.z - self.p_min.z).max(0f32);
        dx * dy * dz
    }

    pub fn max_extent(&self) -> usize {
        let d = &(self.p_max) - &(self.p_min);
        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    pub fn lerp_point(&self, tx: f32, ty: f32, tz: f32) -> Point {
        Point::new_with(
            self.p_min.x.lerp(&self.p_max.x, tx),
            self.p_min.y.lerp(&self.p_max.y, ty),
            self.p_min.z.lerp(&self.p_max.z, tz))
    }

    pub fn offset(&self, p: &Point) -> Vector {
        if self.empty() {
            Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY)
        } else {
            Vector::new_with(
                (p.x - self.p_min.x) / (self.p_max.x - self.p_min.x),
                (p.y - self.p_min.y) / (self.p_max.y - self.p_min.y),
                (p.z - self.p_min.z) / (self.p_max.z - self.p_min.z))
        }
    }

    pub fn bounding_sphere(&self) -> (Point, f32) {
        let c = 0.5f32 * (&self.p_min + &self.p_max);
        let r =
            if !self.inside(&c) { 0f32 } else {
                c.distance(&self.p_max)
            };
        (c, r)
    }
}

impl ::std::ops::Index<usize> for BBox {
    type Output = Point;
    fn index(&self, index: usize) -> &Point {
        match index {
            0 => &self.p_min,
            1 => &self.p_max,
            _ => panic!("Error - BBox index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<usize> for BBox {
    fn index_mut(&mut self, index: usize) -> &mut Point {
        match index {
            0 => &mut self.p_min,
            1 => &mut self.p_max,
            _ => panic!("Error - BBox index out of bounds!")
        }
    }
}

impl<'a> ::std::convert::From<&'a Point> for BBox {
    fn from(pt: &'a Point) -> BBox { BBox::new_with(pt.clone(), pt.clone()) }
}

impl ::std::convert::From<Point> for BBox {
    fn from(pt: Point) -> BBox { BBox::new_with(pt.clone(), pt) }
}

impl Union<Point> for BBox {
    fn union(&self, pt: &Point) -> BBox {
        let p_min = Point::new_with(
            self.p_min.x.min(pt.x),
            self.p_min.y.min(pt.y),
            self.p_min.z.min(pt.z));
        let p_max = Point::new_with(
            self.p_max.x.max(pt.x),
            self.p_max.y.max(pt.y),
            self.p_max.z.max(pt.z));

        BBox::new_with(p_min, p_max)
    }
}

impl Union for BBox {
    fn union(&self, bbox: &BBox) -> BBox {
        let p_min = Point::new_with(
            self.p_min.x.min(bbox.p_min.x),
            self.p_min.y.min(bbox.p_min.y),
            self.p_min.z.min(bbox.p_min.z));
        let p_max = Point::new_with(
            self.p_max.x.max(bbox.p_max.x),
            self.p_max.y.max(bbox.p_max.y),
            self.p_max.z.max(bbox.p_max.z));

        BBox::new_with(p_min, p_max)
    }
}

impl Intersectable<(f32, f32)> for BBox {
    fn intersect(&self, r: &Ray) -> Option<(f32, f32)> {
        let mut t0 = r.mint();
        let mut t1 = r.maxt();

        for i in 0..3 {
            let inv_ray_dir = 1f32 / r.d[i];
            let (t_near, t_far) = {
                let mut t_a = (self.p_min[i] - r.o[i]) * inv_ray_dir;
                let mut t_b = (self.p_max[i] - r.o[i]) * inv_ray_dir;

                if t_a > t_b { ::std::mem::swap(&mut t_a, &mut t_b); }

                (t_a, t_b)
            };

            t0 = t_near.max(t0);
            t1 = t_far.min(t1);

            if t0 > t1 { return None; }
        }

        Some((t0, t1))
    }
}

impl HasBounds for BBox {
    fn world_bound(&self) -> BBox { self.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::point::Point;
    use geometry::normal::Normalize;
    use geometry::vector::Vector;
    use intersection::Intersectable;
    use ray::Ray;
    use std::f32;

    #[test]
    fn it_creates_empty_bbox() {
        let bbox = BBox {
            p_min: Point::new_with(f32::MAX, f32::MAX, f32::MAX),
            p_max: Point::new_with(-f32::MAX, -f32::MAX, -f32::MAX)
        };
        assert_eq!(bbox, BBox::new());
    }

    #[test]
    fn it_creates_a_box_with_specified_points() {
        let pmin = Point::new_with(1f32, 2f32, 4f32);
        let pmax = Point::new_with(2f32, 3f32, 5f32);
        let bbox = BBox {
            p_min: pmin.clone(),
            p_max: pmax.clone()
        };
        assert_eq!(bbox, BBox::new_with(pmin.clone(), pmax.clone()));

        // Test to see if bbox constructor does any checking to see whether
        // min and max are properly opposed. This lets us supply bounded
        // empty bounding boxes.
        assert!(BBox::new_with(pmax, pmin).ne(&bbox))
    }

    #[test]
    fn it_knows_when_its_empty() {
        assert!(BBox::new().empty());

        assert!(BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(0f32, 0f32, 0f32)).empty());

        assert!(BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 0f32, 4f32)).empty());

        assert!(BBox::new_with(
            Point::new_with(2f32, 0f32, 0f32),
            Point::new_with(1f32, 3f32, 4f32)).empty());

        assert!(BBox::new_with(
            Point::new_with(-2f32, 0f32, 0f32),
            Point::new_with(1f32, -3f32, 4f32)).empty());

        assert!(!(BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 3f32, 4f32)).empty()));
    }

    #[test]
    fn it_properly_overlaps_with_others() {
        // Empty bboxes don't overlap
        assert!(!(BBox::new().overlaps(&BBox::new())));

        let bbox_1 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        let bbox_2 = BBox::new_with(
            Point::new_with(0.5f32, 0.5f32, 0.5f32),
            Point::new_with(1.5f32, 1.5f32, 1.5f32));
        assert!(bbox_1.overlaps(&bbox_2));

        // Only overlap at a point
        let bbox_3 = BBox::new_with(
            Point::new_with(1f32, 1f32, 1f32),
            Point::new_with(2f32, 2f32, 2f32));
        assert!(bbox_1.overlaps(&bbox_3));
        assert!(bbox_2.overlaps(&bbox_3));

        // Don't overlap
        let bbox_4 = BBox::new_with(
            Point::new_with(1.5f32, 1.5f32, 1.5f32),
            Point::new_with(2.5f32, 2.5f32, 2.5f32));
        assert!(!(bbox_1.overlaps(&bbox_4)));
        assert!(bbox_2.overlaps(&bbox_4));
        assert!(bbox_3.overlaps(&bbox_4));
    }

    #[test]
    fn it_knows_when_points_are_inside() {
        let bbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        let p1 = Point::new_with(0f32, 0f32, 0f32);
        let p2 = Point::new_with(0f32, 1f32, 0f32);
        let p3 = Point::new_with(0f32, 0.1f32, 0.5f32);
        let p4 = Point::new_with(0.2f32, 0.6f32, 0.1289736f32);
        let p5 = Point::new_with(1.2f32, 0.6f32, 0.1289736f32);
        let p6 = Point::new_with(-1.2f32, 2.6f32, 16f32);

        assert!(bbox.inside(&p1));
        assert!(bbox.inside(&p2));
        assert!(bbox.inside(&p3));
        assert!(bbox.inside(&p4));
        assert!(!(bbox.inside(&p5)));
        assert!(!(bbox.inside(&p6)));

        assert!(!(BBox::new().inside(&p1)));
        assert!(!(BBox::new().inside(&p2)));
        assert!(!(BBox::new().inside(&p3)));
        assert!(!(BBox::new().inside(&p4)));
        assert!(!(BBox::new().inside(&p5)));
        assert!(!(BBox::new().inside(&p6)));
    }

    #[test]
    fn it_can_expand() {
        let mut bbox = BBox::new_with(
            Point::new_with(2f32, 3f32, 4f32),
            Point::new_with(5f32, 5f32, 4f32));
        let expanded = BBox::new_with(
            Point::new_with(1.5f32, 2.5f32, 3.5f32),
            Point::new_with(5.5f32, 5.5f32, 4.5f32));
        bbox.expand(0.5f32);
        assert_eq!(expanded, bbox);
        bbox.expand(0f32);
        assert_eq!(expanded, bbox);
    }

    #[test]
    fn it_has_a_surface_area() {
        let bbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert_eq!(bbox.surface_area(), 6f32);

        assert_eq!(BBox::new().surface_area(), 0f32);

        let bbox_2 = BBox::new_with(
            Point::new_with(1f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert_eq!(bbox_2.surface_area(), 2f32);

        let bbox_3 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(2f32, 3f32, 4f32));
        assert_eq!(bbox_3.surface_area(), 52f32);
    }

    #[test]
    fn it_has_a_volume() {

        let bbox_2 = BBox::new_with(
            Point::new_with(1f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert_eq!(bbox_2.volume(), 0f32);

        let bbox_3 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(2f32, 3f32, 4f32));
        assert_eq!(bbox_3.volume(), 24f32);
    }

    #[test]
    fn it_has_a_maximum_extent() {
        let bbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert!(bbox.max_extent() == 2);
        assert!(BBox::new().max_extent() == 2);

        let bbox_2 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(2f32, 3f32, 4f32));
        assert_eq!(bbox_2.max_extent(), 2);

        let bbox_3 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(4f32, 5f32, 4f32));
        assert_eq!(bbox_3.max_extent(), 1);

        let bbox_4 = BBox::new_with(
            Point::new_with(0f32, 10f32, 0f32),
            Point::new_with(4f32, 5f32, 4f32));
        assert_eq!(bbox_4.max_extent(), 2);
    }

    #[test]
    fn it_can_lerp_points() {
        let bbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert_eq!(bbox.lerp_point(0.2f32, 0.3f32, 0.4f32),
                   Point::new_with(0.2f32, 0.3f32, 0.4f32));

        let bbox_2 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(0f32, 0f32, 0f32));
        assert_eq!(bbox_2.lerp_point(0.2f32, 0.3f32, 0.4f32),
                   Point::new_with(0f32, 0f32, 0f32));
        assert_eq!(bbox_2.lerp_point(-0.2f32, 3f32, 4f32),
                   Point::new_with(0f32, 0f32, 0f32));
        assert_eq!(bbox_2.lerp_point(0f32, 0f32, 0f32),
                   Point::new_with(0f32, 0f32, 0f32));
        assert_eq!(bbox_2.lerp_point(32f32, -3f32, 1e6f32),
                   Point::new_with(0f32, 0f32, 0f32));
    }

    #[test]
    fn it_can_determine_the_offset_for_points() {
        let bbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert_eq!(bbox.offset(&Point::new_with(0.2f32, 0.3f32, 0.4f32)),
                   Vector::new_with(0.2f32, 0.3f32, 0.4f32));

        let bbox_2 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(0f32, 0f32, 0f32));
        assert_eq!(bbox_2.offset(&Point::new_with(0.2f32, 0.3f32, 0.4f32)),
                   Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY));
        assert_eq!(bbox_2.offset(&Point::new_with(-0.2f32, 3f32, 4f32)),
                   Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY));
        assert_eq!(bbox_2.offset(&Point::new_with(0f32, 0f32, 0f32)),
                   Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY));
        assert_eq!(bbox_2.offset(&Point::new_with(32f32, -3f32, 1e6f32)),
                   Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY));

        assert_eq!(BBox::new().offset(&Point::new_with(0.2f32, 0.3f32, 0.4f32)),
                   Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY));
        assert_eq!(BBox::new().offset(&Point::new_with(-0.2f32, 3f32, 4f32)),
                   Vector::new_with(f32::INFINITY, f32::INFINITY, f32::INFINITY));
    }

    #[test]
    fn it_has_a_bounding_sphere() {
        assert_eq!(BBox::new().bounding_sphere(), (Point::new(), 0f32));
        let bbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(0f32, 0f32, 0f32));
        assert_eq!(bbox.bounding_sphere(), (Point::new(), 0f32));

        let bbox_2 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(1f32, 1f32, 1f32));
        assert_eq!(bbox_2.bounding_sphere(), (Point::new_with(0.5f32, 0.5f32, 0.5f32), 0.75f32.sqrt()));
    }

    #[test]
    fn it_can_be_constructed_from_a_point() {
        let origin = Point::new_with(0f32, 0f32, 0f32);
        let empty_bbox = BBox::new_with(origin.clone(), origin.clone());
        assert_eq!(BBox::from(&origin), empty_bbox);
        assert_eq!(BBox::from(origin), empty_bbox);

        let p = Point::new_with(3f32, 2f32, -1f32);
        let still_empty_bbox = BBox::new_with(p.clone(), p.clone());
        assert_eq!(BBox::from(&p), still_empty_bbox);
        assert_eq!(BBox::from(p), still_empty_bbox);
    }

    #[test]
    fn it_can_be_unioned_with_a_point() {
        let origin = Point::new_with(0f32, 0f32, 0f32);
        let p = Point::new_with(3f32, 2f32, -1f32);
        let empty_bbox = BBox::new_with(origin.clone(), origin.clone());
        let still_empty_bbox = BBox::new_with(p.clone(), p.clone());
        assert_eq!(empty_bbox.union(&p), still_empty_bbox.union(&origin));

        let p2 = Point::new_with(3f32, 0f32, 0f32);
        let bbox = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(1f32, 1f32, 1f32));

        let unioned = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(3f32, 1f32, 1f32));
        assert_eq!(bbox.unioned_with(p2), unioned);
    }

    #[test]
    fn it_can_be_unioned_with_another_bbox() {
        let bbox = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(1f32, 1f32, 1f32));
        let bbox2 = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(3f32, 2f32, -1f32));

        let bbox_unioned = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(3f32, 2f32, 1f32));
        assert_eq!(bbox.union(&bbox2), bbox_unioned);

        let bbox3 = BBox::from(Point::new_with(3f32, 0f32, 0f32));
        let unioned = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(3f32, 1f32, 1f32));
        assert_eq!(bbox.union(&bbox3), unioned);

        assert_eq!(BBox::new().unioned_with_ref(&bbox3), bbox3);
    }

    #[test]
    fn it_can_be_indexed() {
        let mut bbox = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(1f32, 1f32, 1f32));
        let ibbox = BBox::new_with(
            Point::new_with(0f32, 0f32, 0f32),
            Point::new_with(3f32, 2f32, -1f32));

        bbox[0] = ibbox[0].clone();
        bbox[1] = ibbox[1].clone();
        assert_eq!(bbox, ibbox);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_indexed_too_much() {
        let bbox = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(1f32, 1f32, 1f32));
        println!("This should never appear: {:?}", bbox[2]);
    }

    #[test]
    #[should_panic]
    fn it_cant_be_mutably_indexed_too_much_either() {
        let mut bbox = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(1f32, 1f32, 1f32));
        bbox[0] = Point::new();
        println!("This should never appear: {:?}", bbox[14]);
    }

    #[test]
    fn it_can_be_intersected() {
        let simple = BBox::new_with(
            Point::new_with(-1f32, -1f32, -1f32),
            Point::new_with(1f32, 1f32, 1f32));

        assert_eq!(Some((1f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(2.0, 0.0, 0.0), Vector::new_with(-1.0, 0.0, 0.0), 0.0)));

        assert_eq!(Some((1f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(-2.0, 0.0, 0.0), Vector::new_with(1.0, 0.0, 0.0), 0.0)));

        assert_eq!(None, simple.intersect(&Ray::new_with(
            Point::new_with(2.0, 0.0, 0.0), Vector::new_with(1.0, 0.0, 0.0), 0.0)));

        assert_eq!(None, simple.intersect(&Ray::new_with(
            Point::new_with(-2.0, 0.0, 0.0), Vector::new_with(-1.0, 0.0, 0.0), 0.0)));

        assert_eq!(Some((1f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(0.0, 2.0, 0.0), Vector::new_with(0.0, -1.0, 0.0), 0.0)));

        assert_eq!(Some((1f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(0.0, -2.0, 0.0), Vector::new_with(0.0, 1.0, 0.0), 0.0)));

        assert_eq!(None, simple.intersect(&Ray::new_with(
            Point::new_with(0.0, 2.0, 0.0), Vector::new_with(0.0, 1.0, 0.0), 0.0)));

        assert_eq!(None, simple.intersect(&Ray::new_with(
            Point::new_with(0.0, -2.0, 0.0), Vector::new_with(0.0, -1.0, 0.0), 0.0)));

        assert_eq!(Some((1f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(0.0, 0.0, 2.0), Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        assert_eq!(Some((1f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(0.0, 0.0, -2.0), Vector::new_with(0.0, 0.0, 1.0), 0.0)));

        assert_eq!(None, simple.intersect(&Ray::new_with(
            Point::new_with(0.0, 0.0, 2.0), Vector::new_with(0.0, 0.0, 1.0), 0.0)));

        assert_eq!(None, simple.intersect(&Ray::new_with(
            Point::new_with(0.0, 0.0, -2.0), Vector::new_with(0.0, 0.0, -1.0), 0.0)));

        let (diag1, diag2) = simple.intersect(&Ray::new_with(
            Point::new_with(-1f32, -1f32, -1f32), Vector::new_with(1f32, 1f32, 1f32).normalize(), 0.0)).unwrap();
        assert_eq!(diag1, 0f32);
        assert!((diag2 - 12f32.sqrt()).abs() < 1e-6);

        // Graze the box
        let (off1, off2) = simple.intersect(&Ray::new_with(
            Point::new_with(1.5, 0.5, 0.5), Vector::new_with(-0.5, 0.0, 0.5), 0.0)).unwrap();
        assert_eq!(off1, 1.0);
        assert_eq!(off2, 1.0);

        // Graze the box at an angle...
        let (off3, off4) = simple.intersect(&Ray::new_with(
            Point::new_with(1.5, 0.5, 0.5), Vector::new_with(-0.5, -0.5, 0.5), 0.0)).unwrap();
        assert_eq!((off3, off4), (1.0, 1.0));

        // What if we start from inside the box?
        assert_eq!(Some((2f32, 3f32)), simple.intersect(&Ray::new_with(
            Point::new_with(2.0, 0.0, 0.0), Vector::new_with(-1.0, 0.0, 0.0), 2.0)));

        // !FIXME! Maybe add more tests with a different bounding box...
    }
}
