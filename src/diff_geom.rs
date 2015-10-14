use vector::Vector;
use vector::Point;
use vector::Normal;

#[derive(Debug, Copy, Clone)]
pub struct DifferentialGeometry {
    pub p: Point,
    pub nn: Normal,
}
