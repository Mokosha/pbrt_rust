use diff_geom::DifferentialGeometry;

pub trait TextureMapping2D {
    // Returns the s and t coordinates for the point on the given surface,
    // and also returns their differentials:
    //   (s, t, ds/dx, dt/dx, ds/dy, dt/dy)
    fn map(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32);
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct UVMapping2D {
    su: f32,
    sv: f32,
    du: f32,
    dv: f32,
}

impl UVMapping2D {
    pub fn new(_su: f32, _sv: f32, _du: f32, _dv: f32) -> UVMapping2D {
        UVMapping2D { su: _su, sv: _sv, du: _du, dv: _dv }
    }
}

impl TextureMapping2D for UVMapping2D {
    fn map(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32) {
        let s = self.su * dg.u + self.du;
        let t = self.sv * dg.v + self.dv;
        let dsdx = self.su * dg.dudx;
        let dtdx = self.sv * dg.dvdx;
        let dsdy = self.su * dg.dudy;
        let dtdy = self.sv * dg.dvdy;
        (s, t, dsdx, dtdx, dsdy, dtdy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diff_geom::DifferentialGeometry;

    #[test]
    fn it_can_create_uv_mapping() {
        let mapping = UVMapping2D::new(1.0, 1.0, 0.0, 0.0);
        assert_eq!(mapping, UVMapping2D::new(1.0, 1.0, 0.0, 0.0));
    }

    #[test]
    fn uv_mapping_can_map_coords() {
        let identity = UVMapping2D::new(1.0, 1.0, 0.0, 0.0);
        let mut dg = DifferentialGeometry::new();
        assert_eq!(identity.map(&dg), (0.0, 0.0, 0.0, 0.0, 0.0, 0.0));

        dg.u = 0.5;
        dg.v = 0.2;
        assert_eq!(identity.map(&dg), (0.5, 0.2, 0.0, 0.0, 0.0, 0.0));

        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;
        assert_eq!(identity.map(&dg), (0.5, 0.2, 10.0, -1.0, 12.0, 0.0));
    }

    #[test]
    fn uv_mapping_can_scale_coords() {
        let mapping = UVMapping2D::new(2.0, 0.5, 1.0, -0.3);
        let mut dg = DifferentialGeometry::new();
        assert_eq!(mapping.map(&dg), (1.0, -0.3, 0.0, 0.0, 0.0, 0.0));

        dg.u = 0.5;
        dg.v = 0.2;
        let expected_t = 0.2 * 0.5 - 0.3;  // Stupid floating point precision
        assert_eq!(mapping.map(&dg), (2.0, expected_t, 0.0, 0.0, 0.0, 0.0));

        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;
        assert_eq!(mapping.map(&dg), (2.0, expected_t, 20.0, -0.5, 24.0, 0.0));
    }
}
