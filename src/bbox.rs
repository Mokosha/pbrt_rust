#[derive(Debug, Copy, Clone)]
pub struct BBox;

pub fn union(a : &BBox, b : &BBox) -> BBox { BBox }

pub trait HasBounds {
    fn get_bounds(&self) -> BBox;
}

impl HasBounds for BBox {
    fn get_bounds(&self) -> BBox { *self }
}
