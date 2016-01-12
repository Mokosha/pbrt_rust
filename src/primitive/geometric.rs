use bbox::BBox;
use bbox::HasBounds;
use primitive::PrimitiveBase;
use shape::Shape;

#[derive(Clone, Debug, PartialEq)]
pub struct GeometricPrimitive {
    base: PrimitiveBase,
    s: Shape
}

impl GeometricPrimitive {
    pub fn new(_s: Shape) -> GeometricPrimitive {
        GeometricPrimitive {
            base: PrimitiveBase::new(),
            s: _s
        }
    }
}

impl HasBounds for GeometricPrimitive {
    fn world_bound(&self) -> BBox { BBox::new() }
}
