use std::sync::Arc;

use diff_geom::DifferentialGeometry;
use texture::internal::TextureBase;
use texture::Texture;
use utils::Lerp;

#[derive(Clone, Debug)]
pub struct MixTexture<T> where T: Lerp<f32> {
    tex1: Arc<dyn Texture<T>>,
    tex2: Arc<dyn Texture<T>>,
    amount: Arc<dyn Texture<f32>>
}

impl<T> MixTexture<T> where T: Lerp<f32> {
    pub fn new(t1: Arc<dyn Texture<T>>, t2: Arc<dyn Texture<T>>,
               amt: Arc<dyn Texture<f32>>) -> MixTexture<T> {
        MixTexture { tex1: t1, tex2: t2, amount: amt }
    }
}

impl<T> super::internal::TextureBase<T> for MixTexture<T> where T: Lerp<f32> {
    fn eval(&self, dg: &DifferentialGeometry) -> T {
        self.tex1.evaluate(&dg)
            .lerp(&self.tex2.evaluate(&dg), self.amount.evaluate(&dg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texture::ConstantTexture;
    use geometry::vector::Vector;

    #[test]
    fn mix_texture_works() {
        let tex1 = Arc::new(ConstantTexture::new(
            Vector::new_with(1.0, -2.0, 15.0))) as Arc<dyn Texture<Vector>>;
        let tex2 = Arc::new(ConstantTexture::new(
            Vector::new_with(1.0, 2.0, 3.0))) as Arc<dyn Texture<Vector>>;
        let amt = Arc::new(ConstantTexture::new(0.75f32)) as Arc<dyn Texture<f32>>;
        let mix_tex = MixTexture::new(tex1, tex2, amt);
        assert_eq!(mix_tex.evaluate(&DifferentialGeometry::new()),
                   Vector::new_with(1.0, 1.0, 6.0));
    }
}
