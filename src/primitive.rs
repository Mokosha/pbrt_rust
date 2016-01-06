use bbox::BBox;
use bbox::HasBounds;
use intersection;
use ray;

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
    
    fn intersect(&self, ray : &ray::Ray,
                 isect : &mut intersection::Intersection) -> bool {
        false
    }    

    fn intersect_p(&self, ray : &ray::Ray) -> bool {
        false
    }
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
}
