use std::sync::Arc;

use bsdf::BSDF;
use bsdf::bssrdf::BSSRDF;
use diff_geom::DifferentialGeometry;
use geometry::normal::Normal;
use geometry::vector::Vector;
use primitive::Primitive;
use ray::Ray;
use ray::RayDifferential;
use spectrum::Spectrum;
use transform::transform::Transform;

#[derive(Debug)]
pub struct Intersection {
    pub dg: DifferentialGeometry,
    pub primitive: Option<Arc<Primitive>>, // !FIXME! This shouldn't be an option.
    pub world_to_object: Transform,
    pub object_to_world: Transform,
    pub shape_id: usize,
    pub primitive_id: usize,
    pub ray_epsilon: f32,
}

impl Intersection {
    pub fn new_with(_dg: DifferentialGeometry, w2o: Transform,
                    o2w: Transform, sid: usize, pid: usize,
                    ray_eps: f32) -> Intersection {
        Intersection {
            dg: _dg.clone(),
            primitive: None,
            world_to_object: w2o,
            object_to_world: o2w,
            shape_id: sid,
            primitive_id: pid,
            ray_epsilon: ray_eps,
        }
    }

    pub fn get_bsdf(&self, ray: &RayDifferential) -> Option<BSDF> {
        let mut new_dg = self.dg.clone();
        new_dg.compute_differentials(ray);
        match self.primitive {
            None => None,
            Some(ref p) => p.get_bsdf(new_dg, &self.object_to_world)
        }
    }

    pub fn get_bssrdf(&self, ray: &RayDifferential) -> Option<BSSRDF> {
        let mut new_dg = self.dg.clone();
        new_dg.compute_differentials(ray);
        match self.primitive {
            None => None,
            Some(ref p) => p.get_bssrdf(new_dg, &self.object_to_world)
        }
    }

    pub fn le(&self, dir: &Vector) -> Spectrum { Spectrum::from(0f32) }
}

pub trait Intersectable<T = Intersection> {
    fn intersect(&self, r: &Ray) -> Option<T>;
    fn intersect_p(&self, r: &Ray) -> bool {
        self.intersect(r).is_some()
    }
}
