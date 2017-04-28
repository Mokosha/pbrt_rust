pub mod mapping2d;
pub mod mapping3d;
pub mod mix;
pub mod bilerp;

use std::sync::Arc;
use std::ops::Deref;
use std::ops::Mul;
use std::fmt::Debug;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;

mod internal {
    use super::*;

    pub trait TextureBase<T> {
        fn eval(&self, &DifferentialGeometry) -> T;
    }

    impl<U, T> TextureBase<T> for U where U: Deref<Target = Texture<T>> {
        fn eval(&self, dg: &DifferentialGeometry) -> T {
            self.deref().evaluate(&dg)
        }
    }
}

pub trait Texture<T>: Debug + Send + Sync + internal::TextureBase<T> {
    fn evaluate(&self, &DifferentialGeometry) -> T;
}

impl<U, T> Texture<T> for U where U:
Debug + Send + Sync + internal::TextureBase<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T {
        self.eval(&dg)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ConstantTexture<T: Clone> {
    value: T
}

impl<T: Clone> ConstantTexture<T> {
    pub fn new(t: T) -> ConstantTexture<T> {
        ConstantTexture { value: t }
    }
}

impl<T> internal::TextureBase<T> for ConstantTexture<T> where T: Clone {
    fn eval(&self, _: &DifferentialGeometry) -> T {
        self.value.clone()
    }
}

#[derive(Clone, Debug)]
pub struct ScaleTexture<T, U> {
    tex1: Arc<Texture<T>>,
    tex2: Arc<Texture<U>>
}

impl<T, U> ScaleTexture<T, U> {
    pub fn new(t1: Arc<Texture<T>>, t2: Arc<Texture<U>>) -> ScaleTexture<T, U> {
        ScaleTexture { tex1: t1, tex2: t2 }
    }
}

impl<T, U, V> internal::TextureBase<V> for ScaleTexture<T, U> where T:
Mul<U, Output=V> {
    fn eval(&self, dg: &DifferentialGeometry) -> V {
        self.tex1.evaluate(&dg) * self.tex2.evaluate(&dg)
    }
}

mod tests {
    use super::*;
    use geometry::vector::Vector;

    #[test]
    fn const_texture_works() {
        let tex = ConstantTexture::new(32u32);
        assert_eq!(tex.evaluate(&DifferentialGeometry::new()), 32u32);
    }

    #[test]
    fn scale_texture_works() {
        let tex1 = Arc::new(ConstantTexture::new(2.0f32)) as Arc<Texture<f32>>;
        let tex2 = Arc::new(ConstantTexture::new(Vector::new_with(1.0, 2.0, 3.0)))
            as Arc<Texture<Vector>>;
        let scale_tex = ScaleTexture::new(tex1, tex2);
        assert_eq!(scale_tex.evaluate(&DifferentialGeometry::new()),
                   Vector::new_with(2.0, 4.0, 6.0));
    }
}
