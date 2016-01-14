mod geometric;
mod transformed;

use area_light::AreaLight;
use bbox::BBox;
use bbox::HasBounds;
use bsdf::BSDF;
use bsdf::BSSRDF;
use diff_geom::DifferentialGeometry;
use intersection::Intersectable;
use intersection::Intersection;
use material::Material;
use ray::Ray;
use shape::Shape;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;

use primitive::geometric::GeometricPrimitive;
use primitive::transformed::TransformedPrimitive;

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

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
pub enum Primitive {
    Geometric(GeometricPrimitive),
    Transformed(TransformedPrimitive)
}

impl Primitive {
    pub fn geometric(s: Shape) -> Primitive {
        Primitive::Geometric(GeometricPrimitive::new(s, Material))
    }

    pub fn transformed(p: Arc<Primitive>, xf: AnimatedTransform) -> Primitive {
        Primitive::Transformed(TransformedPrimitive::new(p, xf))
    }

    pub fn area_light(&self) -> Option<AreaLight> {
        match self {
            &Primitive::Geometric(ref p) => p.area_light().clone(),
            _ => panic!("Only geometric primitives may have area lights")
        }
    }

    pub fn get_bsdf<'a>(&'a self, dg: DifferentialGeometry<'a>,
                        o2w: &Transform) -> Option<BSDF<'a>> {
        match self {
            &Primitive::Geometric(ref p) => p.get_bsdf(dg, o2w),
            _ => panic!("Only geometric primitives may have bsdfs")
        }
    }

    pub fn get_bssrdf<'a>(&'a self, dg: DifferentialGeometry<'a>,
                          o2w: &Transform) -> Option<BSSRDF<'a>> {
        match self {
            &Primitive::Geometric(ref p) => p.get_bssrdf(dg, o2w),
            _ => panic!("Only geometric primitives may have bssrdfs")
        }
    }
}

impl HasBounds for Primitive {
    fn world_bound(&self) -> BBox {
        match self {
            &Primitive::Geometric(ref prim) => prim.world_bound(),
            &Primitive::Transformed(ref p) => p.world_bound()
        }
    }
}

impl<'a> Intersectable<'a> for Primitive {
    fn intersect(&'a self, ray : &Ray) -> Option<Intersection<'a>> {
        match self {
            &Primitive::Geometric(ref prim) => {
                prim.intersect(ray).and_then(|mut isect| {
                    isect.primitive = Some(self);
                    Some(isect)
                })
            },
            &Primitive::Transformed(ref prim) => prim.intersect(ray)
        }
    }

    fn intersect_p(&'a self, ray : &Ray) -> bool {
        match self {
            &Primitive::Geometric(ref prim) => prim.intersect_p(ray),
            &Primitive::Transformed(ref prim) => prim.intersect_p(ray)
        }
    }
}

impl Refinable for Primitive {
    fn refine(self) -> Vec<Primitive> {
        match self {
            Primitive::Geometric(p) =>
                p.refine().iter().cloned().map(Primitive::Geometric).collect(),
            Primitive::Transformed(_) =>
                panic!("Transformed primitive should already be refined!")
        }
    }

    fn is_refined(&self) -> bool {
        match self {
            &Primitive::Geometric(ref p) => p.is_refined(),
            &Primitive::Transformed(ref p) => {
                assert!(p.primitive().is_refined());
                true
            }
        }
    }
}

impl FullyRefinable for Primitive { }
