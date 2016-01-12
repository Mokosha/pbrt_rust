use area_light::AreaLight;
use bbox::BBox;
use bbox::HasBounds;
use bsdf::BSDF;
use bsdf::BSSDF;
use intersection::Intersectable;
use intersection::Intersection;
use ray::Ray;
use shape::Shape;

use std::sync::atomic::AtomicUsize;

#[derive(Clone, Debug)]
pub struct PrimitiveBase {
    pub prim_id: usize
}

static NEXT_PRIM_ID: AtomicUsize = ::std::sync::atomic::ATOMIC_USIZE_INIT;

impl PrimitiveBase {
    pub fn new() -> PrimitiveBase { PrimitiveBase {
        prim_id: NEXT_PRIM_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed) } }
}

impl ::std::cmp::PartialEq for PrimitiveBase {
    fn eq(&self, _: &PrimitiveBase) -> bool { true }
}

pub trait Refinable<T = Self> {
    fn is_refined(&self) -> bool;
    fn refine(self) -> Vec<T>;
}

pub trait FullyRefinable : Refinable<Self>+Sized {
    fn fully_refine(self) -> Vec<Self> {
        let mut todo = self.refine();
        let mut done = Vec::new();

        while let Some(x) = todo.pop() {
            if x.is_refined() {
                done.push(x);
            } else {
                let mut rx = x.refine();
                todo.append(&mut rx);
            }
        }

        done
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Geometric(GeometricPrimitive)
}

impl Primitive {
    pub fn geometric(s: Shape) -> Primitive {
        Primitive::Geometric(GeometricPrimitive::new(s))
    }

    pub fn area_light(&self) -> Option<AreaLight> {
        None
    }

    pub fn get_bsdf<'a>(&'a self) -> Option<BSDF<'a>> {
        None
    }

    pub fn get_bssdf<'a>(&'a self) -> Option<BSSDF<'a>> {
        None
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



