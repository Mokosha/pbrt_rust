mod aggregates;
mod geometric;
mod transformed;

use area_light::AreaLight;
use bbox::BBox;
use bbox::HasBounds;
use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
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
use primitive::aggregates::Aggregate;

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

#[derive(Clone, Debug)] // , PartialEq)]
enum Prim {
    Geometric(GeometricPrimitive),
    Transformed(TransformedPrimitive),
    Aggregate(Aggregate)
}

#[derive(Clone, Debug)] // , PartialEq)]
pub struct Primitive {
    base: PrimitiveBase,
    prim: Arc<Prim>
}

impl Primitive {
    pub fn geometric(s: Shape) -> Primitive {
        Primitive {
            base: PrimitiveBase::new(),
            prim: Arc::new(Prim::Geometric(
                GeometricPrimitive::new(s, Material::broken())))
        }
    }

    pub fn transformed(p: Arc<Primitive>, xf: AnimatedTransform) -> Primitive {
        Primitive {
            base: PrimitiveBase::new(),
            prim: Arc::new(Prim::Transformed(TransformedPrimitive::new(p, xf)))
        }
    }

    pub fn grid(p: Vec<Primitive>, refine_immediately: bool) -> Primitive {
        Primitive {
            base: PrimitiveBase::new(),
            prim: Arc::new(Prim::Aggregate(Aggregate::grid(p, refine_immediately)))
        }
    }

    pub fn get_id(&self) -> usize { self.base.prim_id }

    pub fn area_light(&self) -> Option<AreaLight> {
        match self.prim.as_ref() {
            &Prim::Geometric(ref p) => p.area_light().clone(),
            _ => panic!("Only geometric primitives may have area lights")
        }
    }

    pub fn get_bsdf(&self, dg: DifferentialGeometry,
                    o2w: &Transform) -> Option<BSDF> {
        match self.prim.as_ref() {
            &Prim::Geometric(ref p) => p.get_bsdf(dg, o2w),
            _ => panic!("Only geometric primitives may have bsdfs")
        }
    }

    pub fn get_bssrdf(&self, dg: DifferentialGeometry,
                      o2w: &Transform) -> Option<BSSRDF> {
        match self.prim.as_ref() {
            &Prim::Geometric(ref p) => p.get_bssrdf(dg, o2w),
            _ => panic!("Only geometric primitives may have bssrdfs")
        }
    }
}

impl HasBounds for Primitive {
    fn world_bound(&self) -> BBox {
        match self.prim.as_ref() {
            &Prim::Geometric(ref prim) => prim.world_bound(),
            &Prim::Transformed(ref p) => p.world_bound(),
            &Prim::Aggregate(ref a) => a.world_bound()
        }
    }
}

impl Intersectable for Primitive {
    fn intersect(&self, ray : &Ray) -> Option<Intersection> {
        match self.prim.as_ref() {
            &Prim::Geometric(ref prim) => {
                prim.intersect(ray).and_then(|mut isect| {
                    isect.primitive = Some(Arc::new(self.clone()));
                    Some(isect)
                })
            },
            &Prim::Transformed(ref prim) => prim.intersect(ray),
            &Prim::Aggregate(ref a) => a.intersect(ray)
        }.and_then(|mut isect| {
            isect.primitive_id = self.base.prim_id;
            Some(isect)
        })
    }

    fn intersect_p(&self, ray : &Ray) -> bool {
        match self.prim.as_ref() {
            &Prim::Geometric(ref prim) => prim.intersect_p(ray),
            &Prim::Transformed(ref prim) => prim.intersect_p(ray),
            &Prim::Aggregate(ref a) => a.intersect_p(ray)
        }
    }
}

impl Refinable for Primitive {
    fn refine(self) -> Vec<Primitive> {
        if self.is_refined() {
            return vec![self];
        }

        let prim = match Arc::try_unwrap(self.prim) {
            Ok(p) => p,
            Err(pr_ref) => pr_ref.as_ref().clone()
        };

        let prims = match prim {
            Prim::Geometric(p) =>
                p.refine().iter().cloned().map(Prim::Geometric).collect(),
            Prim::Transformed(_) =>
                panic!("Transformed primitive should already be refined!"),
            Prim::Aggregate(a) => vec![Prim::Aggregate(a)]
        };

        prims.into_iter().map(|p| {
            Primitive {
                base: PrimitiveBase::new(),
                prim: Arc::new(p)
            }
        }).collect()
    }

    fn is_refined(&self) -> bool {
        match self.prim.as_ref() {
            &Prim::Geometric(ref p) => p.is_refined(),
            &Prim::Transformed(ref p) => {
                assert!(p.primitive().is_refined());
                true
            }
            &Prim::Aggregate(_) => true
        }
    }
}

impl FullyRefinable for Primitive { }
