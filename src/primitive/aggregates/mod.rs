mod grid;
mod bvh;

use bbox::BBox;
use bbox::HasBounds;
use intersection::Intersectable;
use intersection::Intersection;
use primitive::Primitive;
use primitive::aggregates::grid::GridAccelerator;
use primitive::aggregates::bvh::BVHAccelerator;
use ray::Ray;

#[derive(Clone, Debug)]
pub enum Aggregate {
    Grid(GridAccelerator),
    BVH(BVHAccelerator)
}

impl Aggregate {
    pub fn grid(p: Vec<Primitive>, refine_immediately: bool) -> Aggregate {
        Aggregate::Grid(GridAccelerator::new(p, refine_immediately))
    }

    pub fn bvh(p: Vec<Primitive>, max_prims: usize, sm: &'static str) -> Aggregate {
        Aggregate::BVH(BVHAccelerator::new(p, max_prims, sm))
    }
}

impl HasBounds for Aggregate {
    fn world_bound(&self) -> BBox {
        match self {
            &Aggregate::Grid(ref ga) => ga.world_bound(),
            &Aggregate::BVH(ref bvh) => bvh.world_bound()
        }
    }
}

impl Intersectable for Aggregate {
    fn intersect(&self, ray : &Ray) -> Option<Intersection> {
        match self {
            &Aggregate::Grid(ref g) => g.intersect(ray),
            &Aggregate::BVH(ref bvh) => bvh.intersect(ray)
        }
    }

    fn intersect_p(&self, ray : &Ray) -> bool {
        match self {
            &Aggregate::Grid(ref g) => g.intersect_p(ray),
            &Aggregate::BVH(ref bvh) => bvh.intersect_p(ray)
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::vector::Vector;
    use primitive::Primitive;
    use shape::Shape;
    use transform::transform::Transform;

    pub fn get_spheres() -> Vec<Primitive> { vec![
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 0.0, 0.0)),
            Transform::translate(&Vector::new_with(0.0, 0.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 0.0, 0.0)),
            Transform::translate(&Vector::new_with(-2.0, 0.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 2.0, 0.0)),
            Transform::translate(&Vector::new_with(0.0, -2.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 2.0, 0.0)),
            Transform::translate(&Vector::new_with(-2.0, -2.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 0.0, 2.0)),
            Transform::translate(&Vector::new_with(0.0, 0.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 0.0, 2.0)),
            Transform::translate(&Vector::new_with(-2.0, 0.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 2.0, 2.0)),
            Transform::translate(&Vector::new_with(0.0, -2.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 2.0, 2.0)),
            Transform::translate(&Vector::new_with(-2.0, -2.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0))]
    }
}
