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
    use super::*;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use intersection::Intersectable;
    use primitive::Primitive;
    use ray::Ray;
    use shape::Shape;
    use transform::transform::Transform;

    pub fn sphere_at(v: Vector) -> Primitive {
        Primitive::geometric(Shape::sphere(
            Transform::translate(&v), Transform::translate(&(-v)),
            false, 1.0, -1.0, 1.0, 360.0))
    }

    pub fn get_spheres() -> Vec<Primitive> {
        vec![ sphere_at(Vector::new_with(0.0, 0.0, 0.0)),
              sphere_at(Vector::new_with(2.0, 0.0, 0.0)),
              sphere_at(Vector::new_with(0.0, 2.0, 0.0)),
              sphere_at(Vector::new_with(2.0, 2.0, 0.0)),
              sphere_at(Vector::new_with(0.0, 0.0, 2.0)),
              sphere_at(Vector::new_with(2.0, 0.0, 2.0)),
              sphere_at(Vector::new_with(0.0, 2.0, 2.0)),
              sphere_at(Vector::new_with(2.0, 2.0, 2.0))]
    }

    fn test_intersection<T>(agg_factory: T)
        where T : FnOnce(Vec<Primitive>) -> Aggregate
    {
        let spheres = get_spheres();
        let ids: Vec<_> = spheres.iter().map(|p| { p.get_id() }).collect();
        let agg = agg_factory(spheres);

        let mut r = Ray::new_with(Point::new_with(0.0, 0.0, -1.0),
                                  Vector::new_with(0.0, 0.0, 1.0), 0.0);
        assert_eq!(agg.intersect(&r).unwrap().primitive_id, ids[0]);

        r = Ray::new_with(Point::new_with(-1.0, 0.0, 2.0),
                          Vector::new_with(1.0, 0.0, 0.0), 0.0);
        assert_eq!(agg.intersect(&r).unwrap().primitive_id, ids[4]);

        // Shoot a ray through the hold in between four spheres and see
        // if it hits the sphere behind them...
        r = Ray::new_with(Point::new_with(4.0, 0.0, 0.0),
                          Vector::new_with(-2.0, 1.0, 1.0), 0.0);
        assert_eq!(agg.intersect(&r).unwrap().primitive_id, ids[6]);
    }

    #[test]
    fn grids_can_intersect_with_rays() {
        test_intersection(|ps| Aggregate::grid(ps, false));
    }

    #[test]
    fn bvhs_can_intersect_with_rays_with_sah() {
        test_intersection(|ps| Aggregate::bvh(ps, 1, "sah"));
    }

    #[test]
    fn bvhs_can_intersect_with_rays_with_middle() {
        test_intersection(|ps| Aggregate::bvh(ps, 1, "middle"));
    }

    #[test]
    fn bvhs_can_intersect_with_rays_with_equal() {
        test_intersection(|ps| Aggregate::bvh(ps, 1, "equal"));
    }
}
