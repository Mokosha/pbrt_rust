use geometry::normal::Normal;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Cross;
use geometry::vector::Vector;
use shape::ShapeBase;
use ray::RayDifferential;

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
            dudx: 0.0,
            dudy: 0.0,
            dvdx: 0.0,
            dvdy: 0.0,
        }
    }

    pub fn compute_differentials(&mut self, ray: &RayDifferential) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
