use bbox::BBox;
use diff_geom::DifferentialGeometry;
use geometry::point::Point;
use ray::Ray;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

use std::sync::atomic::AtomicIsize;

#[derive(Debug, Clone)]
pub struct Shape {
    pub object2world: Transform,
    pub world2object: Transform,
    pub reverse_orientation: bool,
    pub transform_swaps_handedness: bool,
    pub shape_id: isize
}

static next_shape_id: AtomicIsize = ::std::sync::atomic::ATOMIC_ISIZE_INIT;

impl Shape {
    pub fn new(o2w: &Transform, w2o: &Transform, ro: bool) -> Shape {
        Shape {
            object2world: o2w.clone(),
            world2object: w2o.clone(),
            reverse_orientation: ro,
            transform_swaps_handedness: o2w.swaps_handedness(),
            shape_id: next_shape_id.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed)
        }
    }
}

pub trait IsShape {
    fn get_shape<'a>(&'a self) -> &'a Shape;

    fn object_bound(&self) -> BBox;

    fn world_bound(&self) -> BBox {
        let data = self.get_shape();
        data.object2world.xf(self.object_bound())
    }

    // Default is all shapes can intersect..
    fn can_intersect(&self) -> bool { true }

    fn refine<T>(&self) -> Vec<T> where T : IsShape {
        panic!("Refine not implemented for shape!");
    }

    fn intersect(&self, r: &Ray) ->
        // hit, tHit, rayEpsilon, intersection information
        (bool, Option<f32>, Option<f32>, Option<DifferentialGeometry>) {
            panic!("intersect not implemented for shape!");
            (false, None, None, None)
        }

    fn intersect_p(&self, r: &Ray) -> bool {
        panic!("intersect not implemented for shape!");
        false
    }

    fn get_shading_geometry<'a>(&self, _: &Transform, dg: &DifferentialGeometry<'a>) ->
        DifferentialGeometry<'a> {
            dg.clone()
        }

    fn area(&self) -> f32 {
        panic!("area not implemented for shape!");
        0f32
    }
}
