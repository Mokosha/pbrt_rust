use intersection;
use ray;
use bbox;

pub struct Primitive;

impl Primitive {
    pub fn intersect(&self, ray : &ray::Ray,
                 isect : &mut intersection::Intersection) -> bool {
        false
    }    

    pub fn intersect_p(&self, ray : &ray::Ray) -> bool {
        false
    }
}

impl bbox::HasBounds for Primitive {
    fn get_bounds(&self) -> bbox::BBox { bbox::BBox::new() }
}
