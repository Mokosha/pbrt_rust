// Make mod file for primitives



use bbox::BBox;
use bbox::HasBounds;
use intersection::Intersectable;
use intersection::Intersection;
use ray::Ray;
use shape::shape::Shape;

use std::sync::atomic::AtomicIsize;

pub struct PrimitiveBase {
    pub prim_id: isize
}

static NEXT_PRIM_ID: AtomicIsize = ::std::sync::atomic::ATOMIC_ISIZE_INIT;

impl PrimitiveBase {
    pub fn new() -> PrimitiveBase { PrimitiveBase {
        prim_id: NEXT_PRIM_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed) } }
}

pub trait Refinable<T = Self> {
    fn is_refined(&self) -> bool;
    fn refine(self) -> Vec<T>;
}

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

pub enum Primitive {
    Geometric(GeometricPrimitive)
}

impl Primitive {
    pub fn geometric(s: Shape) -> Primitive {
        Primitive::Geometric(GeometricPrimitive::new(s))
    }
}

impl HasBounds for Primitive {
    fn world_bound(&self) -> BBox {
        match self {
            &Primitive::Geometric(ref prim) => { prim.world_bound() }
        }
    }
}

impl<'a> Intersectable<'a> for Primitive {
    fn intersect(&'a self, ray : &Ray) -> Option<Intersection<'a>> {
        None
    }

    fn intersect_p(&'a self, ray : &Ray) -> bool {
        false
    }
}



