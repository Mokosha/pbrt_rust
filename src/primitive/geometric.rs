use std::sync::Arc;

use area_light::AreaLight;
use bbox::BBox;
use bbox::HasBounds;
use intersection::Intersectable;
use intersection::Intersection;
use material::Material;
use primitive::PrimitiveBase;
use primitive::FullyRefinable;
use primitive::Refinable;
use ray::Ray;
use shape::Shape;

#[derive(Clone, Debug, PartialEq)]
pub struct GeometricPrimitive {
    base: PrimitiveBase,
    s: Shape,
    m: Arc<Material>,
    area_light: Arc<Option<AreaLight>>
}

impl GeometricPrimitive {
    pub fn new(_s: Shape, _m: Material) -> GeometricPrimitive {
        GeometricPrimitive {
            base: PrimitiveBase::new(),
            s: _s,
            m: Arc::new(_m),
            area_light: Arc::new(None)
        }
    }

    pub fn new_lit(_s: Shape, _m: Material, al: AreaLight) -> GeometricPrimitive {
        GeometricPrimitive {
            base: PrimitiveBase::new(),
            s: _s,
            m: Arc::new(_m),
            area_light: Arc::new(Some(al))
        }
    }

    pub fn area_light(&self) -> Option<AreaLight> {
        self.area_light.as_ref().clone()
    }
}

impl<'a> Intersectable<'a> for GeometricPrimitive {
    fn intersect(&'a self, ray : &Ray) -> Option<Intersection<'a>> {
        self.s.intersect(ray).map(|si| {
            ray.set_maxt(si.t_hit);
            Intersection::new_with(
                si.dg,
                self.s.base().world2object.clone(),
                self.s.base().object2world.clone(),
                self.s.base().shape_id,
                self.base.prim_id,
                si.ray_epsilon)
        })
    }

    fn intersect_p(&'a self, ray : &Ray) -> bool {
        self.s.intersect_p(ray)
    }
}

impl HasBounds for GeometricPrimitive {
    fn world_bound(&self) -> BBox { self.s.world_bound() }
}

impl Refinable for GeometricPrimitive {
    fn refine(self) -> Vec<GeometricPrimitive> {
        let GeometricPrimitive { base, s, m, area_light } = self;
        s.refine().iter().cloned().map(|ss| {
            GeometricPrimitive {
                base: PrimitiveBase::new(),
                s: ss,
                m: m.clone(),
                area_light: area_light.clone()
            }
        }).collect()
    }

    fn is_refined(&self) -> bool { self.s.is_refined() }
}

impl FullyRefinable for GeometricPrimitive { }
