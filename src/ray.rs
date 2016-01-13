use std::cell::RefCell;

use geometry::point::Point;
use geometry::vector::Vector;
use std::f32;
use time::Time;

#[derive(Debug, PartialEq, Clone)]
pub struct Ray {
    pub o: Point,
    pub d: Vector,
    pub time: Time,
    pub depth: i32,
    mint: RefCell<f32>,
    maxt: RefCell<f32>
}

impl Ray {
    pub fn new() -> Ray {
        Ray {
            o: Point::new(),
            d: Vector::new(),
            time: Time::from(0f32),
            depth: 0,
            mint: RefCell::new(0.0),
            maxt: RefCell::new(f32::MAX)
        }
    }

    pub fn new_with(origin: Point, dir: Vector, start: f32) -> Ray {
        Ray {
            o: origin,
            d: dir,
            time: Time::from(0f32),
            depth: 0,
            mint: RefCell::new(start),
            maxt: RefCell::new(f32::MAX)
        }
    }

    pub fn into(self, origin: Point, dir: Vector, start: f32) -> Ray {
        Ray {
            o: origin,
            d: dir,
            time: self.time,
            depth: self.depth + 1,
            mint: RefCell::new(start),
            maxt: self.maxt.clone()
        }
    }

    pub fn set_mint(&self, t: f32) { *(self.mint.borrow_mut()) = t; }
    pub fn mint(&self) -> f32 { *(self.mint.borrow()) }

    pub fn set_maxt(&self, t: f32) { *(self.maxt.borrow_mut()) = t; }
    pub fn maxt(&self) -> f32 { *(self.maxt.borrow()) }

    pub fn set_time(&mut self, t: f32) { self.time = Time::from(t) }
    pub fn set_depth(&mut self, d: i32) { self.depth = d }

    pub fn point_at(&self, t: f32) -> Point { &self.o + (&self.d * t) }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RayDifferential {
    pub ray: Ray,
    pub has_differentials: bool,
    pub rx_origin: Point,
    pub ry_origin: Point,
    pub rx_dir: Vector,
    pub ry_dir: Vector
}

impl RayDifferential {
    pub fn new() -> RayDifferential {
        RayDifferential {
            ray: Ray::new(),
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new()
        }
    }

    pub fn new_with(origin: Point, dir: Vector, start: f32) -> RayDifferential {
        RayDifferential {
            ray: Ray::new_with(origin, dir, start),
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new()
        }
    }

    pub fn into(self, origin: Point, dir: Vector, start: f32) -> RayDifferential {
        RayDifferential {
            ray: self.ray.clone().into(origin, dir, start),
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new()
        }
    }

    pub fn scale_differentials(&mut self, s: f32) {
        self.rx_origin = &self.ray.o + (&self.rx_origin - &self.ray.o) * s;
        self.ry_origin = &self.ray.o + (&self.ry_origin - &self.ray.o) * s;
        self.rx_dir = &self.ray.d + (&self.rx_dir - &self.ray.d) * s;
        self.ry_dir = &self.ray.d + (&self.ry_dir - &self.ray.d) * s;
    }

    pub fn point_at(&self, t: f32) -> Point { self.ray.point_at(t) }
}

impl ::std::convert::From<Ray> for RayDifferential {
    fn from(r: Ray) -> RayDifferential {
        RayDifferential {
            ray: r,
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use time::Time;

    #[test]
    fn rays_can_be_created() {
        assert_eq!(Ray::new(), Ray {
            o: Point::new(),
            d: Vector::new(),
            time: Time::from(0.0),
            depth: 0,
            mint: RefCell::new(0.0),
            maxt: RefCell::new(::std::f32::MAX)
        });

        let o = Point::new_with(1.0, 2.0, 3.0);
        let d = Vector::new_with(0.0, -1.0, 0.0);
        let r = Ray::new_with(o.clone(), d.clone(), 2.0);
        assert_eq!(r, Ray {
            o: o.clone(),
            d: d.clone(),
            time: Time::from(0.0),
            depth: 0,
            mint: RefCell::new(2.0),
            maxt: RefCell::new(::std::f32::MAX)
        });

        assert_eq!(r.into(Point::new_with(0.1, 1.0, 10.0),
                          Vector::new_with(1.0, 1.0, 1.0), 1.0),
                   Ray {
                       o: Point::new_with(0.1, 1.0, 10.0),
                       d: Vector::new_with(1.0, 1.0, 1.0),
                       time: Time::from(0.0),
                       depth: 1,
                       mint: RefCell::new(1.0),
                       maxt: RefCell::new(::std::f32::MAX)
                   });
    }

    #[test]
    fn ray_differentials_can_be_created() {
        assert_eq!(RayDifferential::new(), RayDifferential {
            ray: Ray::new(),
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new(),
        });

        let o = Point::new_with(1.0, 2.0, 3.0);
        let d = Vector::new_with(0.0, -1.0, 0.0);
        let rd = RayDifferential::new_with(o.clone(), d.clone(), 2.0);
        assert_eq!(rd, RayDifferential {
            ray: Ray::new_with(o.clone(), d.clone(), 2.0),
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new(),
        });

        let mut r = Ray::new_with(Point::new_with (0.1, 1.0, 10.0),
                              Vector::new_with(1.0, 1.0, 1.0), 1.0);
        r.set_depth(1);
        assert_eq!(rd.into(Point::new_with (0.1, 1.0, 10.0),
                           Vector::new_with(1.0, 1.0, 1.0), 1.0),
                   RayDifferential {
                       ray: r,
                       has_differentials: false,
                       rx_origin: Point::new(),
                       ry_origin: Point::new(),
                       rx_dir: Vector::new(),
                       ry_dir: Vector::new(),
                   });
    }

    #[test]
    fn ray_differentials_can_be_scaled() {
        // !FIXME! I'm not totally sure what this function is supposed
        // to do, so I should revisit this after we cover sample spacing
        // in the chapter on cameras...
    }

    #[test]
    fn they_can_be_sampled() {
        let rd = RayDifferential::new_with(Point::new_with(1.0, 2.0, 3.0),
                                           Vector::new_with(0.0, -2.0, 0.0), 0.0);
        let r = Ray::new_with(Point::new_with(1.0, 2.0, 3.0),
                              Vector::new_with(0.0, -2.0, 0.0), 0.0);

        assert_eq!(rd.point_at(0.0), Point::new_with(1.0, 2.0, 3.0));
        assert_eq!(r.point_at(0.0), Point::new_with(1.0, 2.0, 3.0));

        assert_eq!(rd.point_at(1.0), Point::new_with(1.0, 0.0, 3.0));
        assert_eq!(r.point_at(1.0), Point::new_with(1.0, 0.0, 3.0));

        assert_eq!(rd.point_at(0.5), Point::new_with(1.0, 1.0, 3.0));
        assert_eq!(r.point_at(0.5), Point::new_with(1.0, 1.0, 3.0));
    }
}
