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
