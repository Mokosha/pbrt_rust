pub mod mapping;

use diff_geom::DifferentialGeometry;

#[derive(Clone, Debug, PartialEq)]
pub struct Texture<T: ?Sized + Clone> {
    some_t: T
}

impl<T: ?Sized + Clone> Texture<T> {
    pub fn evaluate(&self, _: &DifferentialGeometry) -> T { self.some_t.clone() }
}
