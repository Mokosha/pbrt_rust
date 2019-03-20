use std::sync::Arc;

use area_light::AreaLight;
use bbox::BBox;
use bbox::HasBounds;
use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
use diff_geom::DifferentialGeometry;
use intersection::Intersectable;
use intersection::Intersection;
use material::Material;
use primitive::FullyRefinable;
use primitive::Refinable;
use ray::Ray;
use shape::Shape;
use transform::transform::Transform;

#[derive(Clone, Debug)]
pub struct GeometricPrimitive {
    s: Shape,
    m: Arc<Material>,
    area_light: Option<Arc<AreaLight>>
}

impl GeometricPrimitive {
    pub fn new(_s: Shape, _m: Arc<Material>) -> GeometricPrimitive {
        GeometricPrimitive {
            s: _s,
            m: _m,
            area_light: None
        }
    }

    pub fn new_lit(_s: Shape, _m: Arc<Material>, al: Arc<AreaLight>) -> GeometricPrimitive {
        GeometricPrimitive {
            s: _s,
            m: _m,
            area_light: Some(al.clone())
        }
    }

    pub fn area_light(&self) -> Option<Arc<AreaLight>> {
        self.area_light.clone()
    }

    pub fn get_bsdf(&self, dg: DifferentialGeometry,
                    o2w: &Transform) -> Option<BSDF> {
        let dgs = self.s.get_shading_geometry(o2w, dg.clone());
        self.m.get_bsdf(dg, dgs)
    }

    pub fn get_bssrdf(&self, dg: DifferentialGeometry,
                      o2w: &Transform) -> Option<BSSRDF> {
        let dgs = self.s.get_shading_geometry(o2w, dg.clone());
        self.m.get_bssrdf(dg, dgs)
    }

    pub fn can_intersect(&self) -> bool { self.s.can_intersect() }
}

impl Intersectable for GeometricPrimitive {
    fn intersect(&self, ray : &Ray) -> Option<Intersection> {
        self.s.intersect(ray).map(|si| {
            ray.set_maxt(si.t_hit);
            Intersection::new_with(
                si.dg,
                self.s.base().world2object.clone(),
                self.s.base().object2world.clone(),
                self.s.base().shape_id,
                0,
                si.ray_epsilon)
        })
    }

    fn intersect_p(&self, ray : &Ray) -> bool {
        self.s.intersect_p(ray)
    }
}

impl HasBounds for GeometricPrimitive {
    fn world_bound(&self) -> BBox { self.s.world_bound() }
}

impl Refinable for GeometricPrimitive {
    fn refine(self) -> Vec<GeometricPrimitive> {
        let GeometricPrimitive { s, m, area_light } = self;
        s.refine().iter().cloned().map(|ss| {
            GeometricPrimitive {
                s: ss,
                m: m.clone(),
                area_light: area_light.clone()
            }
        }).collect()
    }

    fn is_refined(&self) -> bool { self.s.is_refined() }
}

impl FullyRefinable for GeometricPrimitive { }
