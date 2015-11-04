use geometry::Lerp;
use geometry::distance;
use geometry::Point;
use geometry::Vector;
use std::f32;

pub trait Union<T: ?Sized = Self>: Sized {
    fn union(&self, &T) -> Self;
    fn unioned_with(self, v: &T) -> Self {
        self.union(v)
    }
}

#[derive(Debug, Clone)]
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
        let d = &(self.p_max) - &(self.p_min);
        2f32 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    pub fn volume(&self) -> f32 {
        let d = &(self.p_max) - &(self.p_min);
        d.x * d.y * d.z
    }

    pub fn max_extent(&self) -> i32 {
        let d = &(self.p_max) - &(self.p_min);
        if (d.x > d.y && d.x > d.z) {
            0
        } else if (d.y > d.z) {
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
        Vector::new_with(
            (p.x - self.p_min.x) / (self.p_max.x - self.p_min.x),
            (p.y - self.p_min.y) / (self.p_max.y - self.p_min.y),
            (p.z - self.p_min.z) / (self.p_max.z - self.p_min.z))
    }

    pub fn bounding_sphere(&self) -> (Point, f32) {
        let c = 0.5f32 * (&self.p_min + &self.p_max);
        let r =
            if (self.inside(&c)) {
                distance(&c, &self.p_max)
            } else {
                0f32
            };
        (c, r)
    }
}

impl ::std::ops::Index<i32> for BBox {
    type Output = Point;
    fn index<'a>(&'a self, index: i32) -> &'a Point {
        match index {
            0 => &self.p_min,
            1 => &self.p_max,
            _ => panic!("Error - BBox index out of bounds!")
        }
    }
}

impl ::std::ops::IndexMut<i32> for BBox {
    fn index_mut<'a>(&'a mut self, index: i32) -> &'a mut Point {
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

pub trait HasBounds {
    fn get_bounds(&self) -> BBox;
}

impl HasBounds for BBox {
    fn get_bounds(&self) -> BBox { self.clone() }
}
