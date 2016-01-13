use area_light::AreaLight;
use bbox::BBox;
use bbox::HasBounds;
use intersection::Intersectable;
use intersection::Intersection;
use material::Material;
use primitive::PrimitiveBase;
use ray::Ray;
use shape::Shape;

#[derive(Clone, Debug, PartialEq)]
pub struct GeometricPrimitive {
    base: PrimitiveBase,
    s: Shape,
    m: Material,
    area_light: Option<AreaLight>
}

impl GeometricPrimitive {
    pub fn new(_s: Shape, _m: Material) -> GeometricPrimitive {
        GeometricPrimitive {
            base: PrimitiveBase::new(),
            s: _s,
            m: _m,
            area_light: None
        }
    }

    pub fn new_lit(_s: Shape, _m: Material, al: AreaLight) -> GeometricPrimitive {
        GeometricPrimitive {
            base: PrimitiveBase::new(),
            s: _s,
            m: _m,
            area_light: Some(al)
        }
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
    fn world_bound(&self) -> BBox { BBox::new() }
}
