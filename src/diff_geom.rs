use geometry::normal::Normal;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Cross;
use geometry::vector::Dot;
use geometry::vector::Vector;
use shape::ShapeBase;
use ray::RayDifferential;

use utils::solve_linear_system_2x2;

#[derive(Debug, PartialEq, Clone)]
pub struct DifferentialGeometry {
    pub p: Point,
    pub nn: Normal,
    pub u: f32,
    pub v: f32,
    pub shape: Option<ShapeBase>,
    pub dpdu: Vector,
    pub dpdv: Vector,
    pub dndu: Normal,
    pub dndv: Normal,
    pub dpdx: Vector,
    pub dpdy: Vector,
    pub dudx: f32,
    pub dudy: f32,
    pub dvdx: f32,
    pub dvdy: f32,
}

impl DifferentialGeometry {
    pub fn new() -> DifferentialGeometry {
        DifferentialGeometry {
            p: Point::new(),
            nn: Normal::new(),
            u: 0f32,
            v: 0f32,
            shape: None,
            dpdu: Vector::new(),
            dpdv: Vector::new(),
            dndu: Normal::new(),
            dndv: Normal::new(),
            dpdx: Vector::new(),
            dpdy: Vector::new(),
            dudx: 0.0,
            dudy: 0.0,
            dvdx: 0.0,
            dvdy: 0.0,
        }
    }

    pub fn new_with(_p: Point, _dpdu: Vector, _dpdv: Vector,
                    _dndu: Normal, _dndv: Normal, _u: f32, _v: f32,
                    _shape: Option<ShapeBase>) -> DifferentialGeometry {
        let mut norm = _dpdu.cross_with(&_dpdv).normalize();
        if let &Some(ref s) = &_shape {
            if s.reverse_orientation ^ s.transform_swaps_handedness {
                norm = norm * -1f32;
            }
        }

        DifferentialGeometry {
            p: _p,
            nn: Normal::from(norm),
            u: _u,
            v: _v,
            shape: _shape,
            dpdu: _dpdu,
            dpdv: _dpdv,
            dndu: _dndu,
            dndv: _dndv,
            dpdx: Vector::new(),
            dpdy: Vector::new(),
            dudx: 0.0,
            dudy: 0.0,
            dvdx: 0.0,
            dvdy: 0.0,
        }
    }

    pub fn compute_differentials(&mut self, ray: &RayDifferential) {
        if !ray.has_differentials {
            self.dpdx = Vector::new();
            self.dpdy = Vector::new();
            self.dudx = 0.0;
            self.dudy = 0.0;
            self.dvdx = 0.0;
            self.dvdy = 0.0;
            return;
        }

        // Compute auxiliary intersection points with plane
        let nvec = Vector::from(self.nn.clone());
        let d = -(nvec.dot(&Vector::from(self.p.clone())));
        let px = {
            let rxv = Vector::from(ray.rx_origin.clone());
            let tx = {
                let ndrx = -(nvec.dot(&rxv) + d);
                let ndrd = nvec.dot(&ray.rx_dir);
                ndrx / ndrd
            };
            &ray.rx_origin + tx * &ray.rx_dir
        };

        let py = {
            let ryv = Vector::from(ray.ry_origin.clone());
            let ty = {
                let ndry = -(nvec.dot(&ryv) + d);
                let ndrd = nvec.dot(&ray.ry_dir);
                ndry / ndrd
            };
            &ray.ry_origin + ty * &ray.ry_dir
        };
        
        self.dpdx = px - &self.p;
        self.dpdy = py - &self.p;

        // Compute (u, v) offsets at auxiliary points

        // Initialize A, Bx, and By matricies for offset computation
        let axes =
            if self.nn.x.abs() > self.nn.y.abs() &&
            self.nn.x.abs() > self.nn.z.abs() {
                [1, 2]
            } else if self.nn.y.abs() > self.nn.z.abs() {
                [0, 2]
            } else {
                [0, 1]
            };

        // Initialize matrices for chosen projection plane
        let a = [[self.dpdu[axes[0]], self.dpdv[axes[0]]],
                 [self.dpdu[axes[1]], self.dpdv[axes[1]]]];
        let bx = [self.dpdx[axes[0]], self.dpdx[axes[1]]];
        let by = [self.dpdy[axes[0]], self.dpdy[axes[1]]];

        if let Some((x, y)) = solve_linear_system_2x2(a.clone(), bx) {
            self.dudx = x;
            self.dvdx = y;
        } else {
            self.dudx = 0.0;
            self.dvdx = 0.0;
        }

        if let Some((x, y)) = solve_linear_system_2x2(a.clone(), by) {
            self.dudy = x;
            self.dvdy = y;
        } else {
            self.dudy = 0.0;
            self.dvdy = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_can_be_created() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn it_can_compute_differentials() {
        // Try with a different plane example for each axis like on p. 506
        unimplemented!()
    }
}
