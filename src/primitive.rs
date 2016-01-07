use bbox::BBox;
use bbox::HasBounds;
use intersection::Intersection;
use ray::Ray;

use std::sync::atomic::AtomicIsize;

pub struct PrimitiveBase {
    pub prim_id: isize
}

static NEXT_PRIM_ID: AtomicIsize = ::std::sync::atomic::ATOMIC_ISIZE_INIT;

impl PrimitiveBase {
    pub fn new() -> PrimitiveBase { PrimitiveBase {
        prim_id: NEXT_PRIM_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed) } }
}

pub trait Primitive : HasBounds {
    fn new() -> Self;
    fn intersect(&self, ray : &Ray) -> Option<Intersection>;
    fn intersect_p(&self, ray : &Ray) -> bool;
    fn can_intersect(&self) -> bool;
}

pub struct GeometricPrimitive {
    base: PrimitiveBase,
}

impl HasBounds for GeometricPrimitive {
    fn world_bound(&self) -> BBox { BBox::new() }
}

impl Primitive for GeometricPrimitive {
    fn new() -> GeometricPrimitive {
        GeometricPrimitive {
            base: PrimitiveBase::new()
        }
    }

    fn intersect(&self, ray : &Ray) -> Option<Intersection> { None }
    fn intersect_p(&self, ray : &Ray) -> bool { false }
    fn can_intersect(&self) -> bool { false }
}
