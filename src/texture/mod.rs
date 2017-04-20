pub mod mapping2d;
pub mod mapping3d;

use std::sync::Arc;
use std::ops::Deref;
use std::fmt::Debug;
use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;

pub trait TextureBase<T> {
    fn eval(&self, &DifferentialGeometry) -> T;
}

impl<U, T> TextureBase<T> for U where U: Deref<Target = Texture<T>> {
    fn eval(&self, dg: &DifferentialGeometry) -> T {
        self.deref().evaluate(&dg)
    }
}

pub trait Texture<T>: Debug + Send + Sync + TextureBase<T> {
    fn evaluate(&self, &DifferentialGeometry) -> T;
}

impl<U, T> Texture<T> for U where U: Debug + Send + Sync + TextureBase<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T {
        self.eval(&dg)
    }
}
