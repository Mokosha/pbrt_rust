use std::sync::Arc;

use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::internal::TextureBase;
use texture::mapping2d::TextureMapping2D;
use texture::Texture;

#[derive(Debug)]
pub struct UVTexture {
    mapping: Box<TextureMapping2D>
}

impl UVTexture {
    pub fn new(_mapping: Box<TextureMapping2D>) -> UVTexture {
        UVTexture { mapping: _mapping }
    }
}

impl super::internal::TextureBase<Spectrum> for UVTexture {
    fn eval(&self, dg: &DifferentialGeometry) -> Spectrum {
        let (s, t, _, _, _, _) = self.mapping.map(dg);
        let rgb = [s - s.floor(), t - t.floor(), 0.0];
        Spectrum::from_rgb(rgb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diff_geom::DifferentialGeometry;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use texture::mapping2d::PlanarMapping2D;

    #[test]
    fn uv_texture_works() {
        let mapping = Box::new(PlanarMapping2D::new());
        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.25, 0.25, 0.0);

        let uv_tex = UVTexture::new(mapping);
        assert_eq!(uv_tex.evaluate(&dg),
                   Spectrum::from_rgb([dg.p.x, dg.p.y, dg.p.z]));

        dg.p = Point::new_with(1.25, -2.5, 0.0);
        assert_eq!(uv_tex.evaluate(&dg),
                   Spectrum::from_rgb([0.25, 0.5, 0.0]));
    }
}
