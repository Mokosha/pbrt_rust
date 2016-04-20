use std::sync::Arc;

use bsdf::BSDF;
use diff_geom::DifferentialGeometry;
use geometry::normal::Normal;
use geometry::vector::Vector;
use primitive::Primitive;
use ray::Ray;
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
    bsdf: BSDF
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
            bsdf: BSDF::new(_dg, Normal::new(), 1.0)
        }
    }

    pub fn get_bsdf(&self) -> &BSDF { &self.bsdf }
    pub fn le(&self, dir: &Vector) -> Spectrum { Spectrum::from(0f32) }
}

pub trait Intersectable<T = Intersection> {
    fn intersect(&self, &Ray) -> Option<T>;
    fn intersect_p(&self, r: &Ray) -> bool {
        self.intersect(r).is_some()
    }
}
