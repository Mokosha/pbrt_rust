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

pub trait Refinable<'a, T = Self> {
    fn is_refined(&'a self) -> bool;
    fn refine(&'a self) -> Vec<T>;
}

pub struct GeometricPrimitive<'a> {
    base: PrimitiveBase,
    s: Shape<'a>
}

impl<'a> GeometricPrimitive<'a> {
    pub fn new(_s: Shape<'a>) -> GeometricPrimitive<'a> {
        GeometricPrimitive {
            base: PrimitiveBase::new(),
            s: _s
        }
    }
}

impl<'a> HasBounds for GeometricPrimitive<'a> {
    fn world_bound(&self) -> BBox { BBox::new() }
}

pub enum Primitive<'a> {
    Geometric(GeometricPrimitive<'a>)
}

impl<'a> Primitive<'a> {
    pub fn geometric(s: Shape<'a>) -> Primitive<'a> {
        Primitive::Geometric(GeometricPrimitive::new(s))
    }
}

impl<'a> HasBounds for Primitive<'a> {
    fn world_bound(&self) -> BBox {
        match self {
            &Primitive::Geometric(ref prim) => { prim.world_bound() }
        }
    }
}

impl<'a> Intersectable<'a> for Primitive<'a> {
    fn intersect(&'a self, ray : &Ray) -> Option<Intersection<'a>> {
        None
    }

    fn intersect_p(&'a self, ray : &Ray) -> bool {
        false
    }
}



