use std::fmt::Debug;
use std::sync::Arc;

use diff_geom::DifferentialGeometry;
use texture::internal::TextureBase;
use texture::mapping2d::TextureMapping2D;
use utils::Lerp;

#[derive(Clone, Debug)]
pub struct BilerpTexture<T> where T: Lerp<f32> {
    mapping: Arc<TextureMapping2D>,
    v00: T,
    v01: T,
    v10: T,
    v11: T
}

impl<T> BilerpTexture<T> where T: Lerp<f32> {
    fn new<Map: TextureMapping2D + 'static>
        (map: Map, t00: T, t01: T, t10: T, t11: T) -> BilerpTexture<T> {
        BilerpTexture {
            mapping: Arc::new(map),
            v00: t00, v01: t01, v10: t10, v11: t11
        }
    }
}

impl<T> super::internal::TextureBase<T> for BilerpTexture<T> where T: Lerp<f32> {
    fn eval(&self, dg: &DifferentialGeometry) -> T {
        let (s, t, _, _, _, _) = self.mapping.map(&dg);
        let tmp1 = self.v00.lerp(&self.v10, s);
        let tmp2 = self.v01.lerp(&self.v11, s);
        tmp1.lerp(&tmp2, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texture::ConstantTexture;
    use texture::Texture;
    use texture::mapping2d::UVMapping2D;
    use geometry::vector::Vector;

    #[test]
    fn bilerp_texture_works() {
        let bilerp_tex: BilerpTexture<f32> =
            BilerpTexture::new(UVMapping2D::new(), 2.0, 3.0, 1.0, 4.0);
        let mut dg = DifferentialGeometry::new();
        assert_eq!(bilerp_tex.evaluate(&dg), 2.0);

        dg.u = 0.5;
        dg.v = 0.5;
        assert_eq!(bilerp_tex.evaluate(&dg), 2.5);

        dg.u = 1.0;
        assert_eq!(bilerp_tex.evaluate(&dg), 2.5);

        dg.v = 1.0;
        assert_eq!(bilerp_tex.evaluate(&dg), 4.0);
    }
}
