use geometry::point::Point;
use geometry::vector::Vector;
use std::f32;
use time::Time;

#[derive(Debug, Clone)]
pub struct Ray {
    pub o: Point,
    pub d: Vector,
    pub mint: f32,
    pub maxt: f32,
    pub time: Time,
    pub depth: i32
}

impl Ray {
    pub fn new() -> Ray {
        Ray {
            o: Point::new(),
            d: Vector::new(),
            mint: 0f32,
            maxt: f32::MAX,
            time: Time::from(0f32),
            depth: 0
        }
    }

    pub fn new_with(origin: &Point, dir: &Vector, start: f32) -> Ray {
        Ray {
            o: origin.clone(),
            d: dir.clone(),
            mint: start,
            maxt: f32::MAX,
            time: Time::from(0f32),
            depth: 0
        }
    }

    pub fn into(self, origin: &Point, dir: &Vector, start: f32) -> Ray {
        Ray {
            o: origin.clone(),
            d: dir.clone(),
            mint: start,
            maxt: self.maxt,
            time: self.time,
            depth: self.depth + 1
        }
    }

    pub fn set_mint(&mut self, t: f32) { self.mint = t }
    pub fn set_maxt(&mut self, t: f32) { self.maxt = t }
    pub fn set_time(&mut self, t: f32) { self.time = Time::from(t) }
    pub fn set_depth(&mut self, d: i32) { self.depth = d }
}

#[derive(Debug, Clone)]
pub struct RayDifferential {
    pub ray: Ray,
    has_differentials: bool,
    rx_origin: Point,
    ry_origin: Point,
    rx_dir: Vector,
    ry_dir: Vector
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

    pub fn new_with(origin: &Point, dir: &Vector, start: f32) -> RayDifferential {
        RayDifferential {
            ray: Ray::new_with(origin, dir, start),
            has_differentials: false,
            rx_origin: Point::new(),
            ry_origin: Point::new(),
            rx_dir: Vector::new(),
            ry_dir: Vector::new()
        }
    }

    pub fn into(self, origin: &Point, dir: &Vector, start: f32) -> RayDifferential {
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
