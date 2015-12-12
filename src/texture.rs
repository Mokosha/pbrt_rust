#[derive(Clone, Debug, PartialEq)]
pub struct Texture<T: ?Sized> {
    some_t: T
}

pub fn white_float_tex() -> Texture<f32> {
    Texture { some_t: 1.0 }
}
