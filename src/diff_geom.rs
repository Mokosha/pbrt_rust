use geometry::Normal;
use geometry::Normalize;
use geometry::Point;
use geometry::Vector;
use shape::Shape;

#[derive(Debug, Clone)]
pub struct DifferentialGeometry<'a> {
    pub p: Point,
    pub nn: Normal,
    pub u: f32,
    pub v: f32,
    pub shape: Option<&'a Shape>,
    pub dpdu: Vector,
    pub dpdv: Vector,
    pub dndu: Normal,
    pub dndv: Normal
}

impl<'a> DifferentialGeometry<'a> {
    pub fn new() -> DifferentialGeometry<'a> {
        DifferentialGeometry {
            p: Point::new(),
            nn: Normal::new(),
            u: 0f32,
            v: 0f32,
            shape: None,
            dpdu: Vector::new(),
            dpdv: Vector::new(),
            dndu: Normal::new(),
            dndv: Normal::new()
        }
    }

    pub fn new_with(_p: Point, _dpdu: Vector, _dpdv: Vector,
                _dndu: Normal, _dndv: Normal, _u: f32, _v: f32,
                _shape: Option<&'a Shape>) -> DifferentialGeometry<'a> {
        let mut norm = _dpdu.clone().cross(&_dpdv).normalize();
        if let Some(s) = _shape {
            if (s.reverse_orientation ^ s.transform_swaps_handedness) {
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
            dndv: _dndv
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
