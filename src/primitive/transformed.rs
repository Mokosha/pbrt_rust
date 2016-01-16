use std::sync::Arc;

use bbox::BBox;
use bbox::HasBounds;
use geometry::normal::Normalize;
use intersection::Intersectable;
use intersection::Intersection;
use primitive::Primitive;
use primitive::Refinable;
use ray::Ray;
use transform::animated::AnimatedTransform;
use transform::transform::ApplyTransform;

#[derive(Clone, Debug)]  // , PartialEq)]
pub struct TransformedPrimitive {
    prim: Arc<Primitive>,
    xf: AnimatedTransform
}

impl TransformedPrimitive {
    pub fn new(p: Arc<Primitive>, xform: AnimatedTransform) -> TransformedPrimitive {
        assert!(p.is_refined());
        TransformedPrimitive {
            prim: p.clone(),
            xf: xform
        }
    }

    pub fn primitive(&self) -> &Primitive {
        self.prim.as_ref()
    }
}

impl Intersectable for TransformedPrimitive {
    fn intersect(&self, ray : &Ray) -> Option<Intersection> {
        let w2p = self.xf.interpolate(f32::from(ray.time));
        let r = w2p.t(ray);
        self.prim.intersect(&r).and_then(|mut isect| {
            ray.set_maxt(r.maxt());

            isect.world_to_object = &isect.world_to_object * &w2p;
            isect.object_to_world = isect.world_to_object.inverse();

            let prim2world = w2p.invert();
            isect.dg.p = prim2world.t(&isect.dg.p);
            isect.dg.nn = prim2world.t(&isect.dg.nn).normalize();
            isect.dg.dpdu = prim2world.t(&isect.dg.dpdu);
            isect.dg.dpdv = prim2world.t(&isect.dg.dpdv);
            isect.dg.dndu = prim2world.t(&isect.dg.dndu);
            isect.dg.dndv = prim2world.t(&isect.dg.dndv);

            Some(isect)
        })
    }

    fn intersect_p(&self, ray : &Ray) -> bool {
        let w2p = self.xf.interpolate(f32::from(ray.time));
        self.prim.intersect_p(&w2p.t(ray))
    }
}

impl HasBounds for TransformedPrimitive {
    fn world_bound(&self) -> BBox {
        let wb = self.prim.world_bound();
        self.xf.motion_bounds(&wb, true)
    }
}
