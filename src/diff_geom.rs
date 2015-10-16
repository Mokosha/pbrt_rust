use geometry::Vector;
use geometry::Point;
use geometry::Normal;

#[derive(Debug, Clone)]
pub struct DifferentialGeometry {
    pub p: Point,
    pub nn: Normal,
}
