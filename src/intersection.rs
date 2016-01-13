use bsdf::BSDF;
use diff_geom::DifferentialGeometry;
use geometry::vector::Vector;
use primitive::Primitive;
use ray::Ray;
use spectrum::Spectrum;
use transform::transform::Transform;

#[derive(Debug, Clone)]
pub struct Intersection<'a> {
    pub dg: DifferentialGeometry<'a>,
    pub primitive: Option<&'a Primitive>, // !FIXME! This shouldn't be an option.
    pub world_to_object: Transform,
    pub object_to_world: Transform,
    pub shape_id: usize,
    pub primitive_id: usize,
    pub ray_epsilon: f32,
    bsdf: BSDF<'a>
}

impl<'a> Intersection<'a> {
    // !FIXME! We should get rid of this when we generate intersections
    // based on actually intersecting things in the future...
    pub fn new() -> Intersection<'a> {
        Intersection {
            dg: DifferentialGeometry::new(),
            primitive: None,
            world_to_object: Transform::new(),
            object_to_world: Transform::new(),
            shape_id: 0,
            primitive_id: 0,
            ray_epsilon: 0f32,
            bsdf: BSDF::new()
        }
    }

    pub fn new_with(_dg: DifferentialGeometry<'a>, w2o: Transform,
                    o2w: Transform, sid: usize, pid: usize, ray_eps: f32) -> Intersection<'a> {
        Intersection {
            dg: _dg,
            primitive: None,
            world_to_object: w2o,
            object_to_world: o2w,
            shape_id: sid,
            primitive_id: pid,
            ray_epsilon: ray_eps,
            bsdf: BSDF::new()
        }
    }

    pub fn get_bsdf(&self) -> &BSDF<'a> { &self.bsdf }
    pub fn le(&self, dir: &Vector) -> Spectrum { Spectrum::from_value(0f32) }
}

pub trait Intersectable<'a, T = Intersection<'a>> {
    fn intersect(&'a self, ray : &Ray) -> Option<T> { None }
    fn intersect_p(&'a self, ray : &Ray) -> bool { false }
}
