use diff_geom::DifferentialGeometry;
use vector::Normal;
use vector::Point;

pub struct BSDF {
    pub dg_shading: DifferentialGeometry
}

impl BSDF {
    pub fn new() -> BSDF {
        BSDF {
            dg_shading: DifferentialGeometry {
                p: Point,
                nn: Normal
            }
        }
    }
}
