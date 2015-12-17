use diff_geom::DifferentialGeometry;

#[derive(Clone, Debug, PartialEq)]
pub struct Texture<T: ?Sized+Copy> {
    some_t: T
}

impl<T: ?Sized+Copy> Texture<T> {
    pub fn evaluate(&self, _: &DifferentialGeometry) -> T { self.some_t }
}
