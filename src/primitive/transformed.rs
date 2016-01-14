use std::sync::Arc;

use bbox::BBox;
use bbox::HasBounds;
use diff_geom::DifferentialGeometry;
use geometry::normal::Normalize;
use intersection::Intersectable;
use intersection::Intersection;
use material::Material;
use primitive::Primitive;
use primitive::PrimitiveBase;
use primitive::Refinable;
use ray::Ray;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;
use transform::transform::ApplyTransform;

#[derive(Clone, Debug, PartialEq)]
pub struct TransformedPrimitive {
    base: PrimitiveBase,
    prim: Arc<Primitive>,
    xf: AnimatedTransform
}

impl TransformedPrimitive {
    pub fn new(p: Arc<Primitive>, xform: AnimatedTransform) -> TransformedPrimitive {
        assert!(p.is_refined());
        TransformedPrimitive {
            base: PrimitiveBase::new(),
            prim: p.clone(),
            xf: xform
        }
    }

    pub fn primitive<'a>(&'a self) -> &'a Primitive {
        self.prim.as_ref()
    }
}

impl<'a> Intersectable<'a> for TransformedPrimitive {
    fn intersect(&'a self, ray : &Ray) -> Option<Intersection<'a>> {
        let w2p = self.xf.interpolate(f32::from(ray.time));
        let r = w2p.t(ray);
        self.prim.intersect(&r).and_then(|mut isect| {
            ray.set_maxt(r.maxt());
            isect.primitive_id = self.base.prim_id;

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

    fn intersect_p(&'a self, ray : &Ray) -> bool {
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
