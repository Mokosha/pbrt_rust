use std::rc::Rc;
use std::sync::atomic::AtomicIsize;

use bbox::BBox;
use bbox::HasBounds;
use diff_geom::DifferentialGeometry;
use geometry::normal::Normal;
use geometry::point::Point;
use geometry::vector::Vector;
use primitive::Refinable;
use texture::Texture;
use transform::transform::Transform;

use shape::sphere::Sphere;
use shape::cylinder::Cylinder;
use shape::disk::Disk;
use shape::mesh::Triangle;
use shape::mesh::Mesh;
use shape::loopsubdiv::LoopSubdiv;

#[derive(Debug, Clone)]
pub struct ShapeBase {
    pub object2world: Transform,
    pub world2object: Transform,
    pub reverse_orientation: bool,
    pub transform_swaps_handedness: bool,
    pub shape_id: isize
}

static NEXT_SHAPE_ID: AtomicIsize = ::std::sync::atomic::ATOMIC_ISIZE_INIT;

impl ShapeBase {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool) -> ShapeBase {
        let swap = o2w.swaps_handedness();
        ShapeBase {
            object2world: o2w,
            world2object: w2o,
            reverse_orientation: ro,
            transform_swaps_handedness: swap,
            shape_id: NEXT_SHAPE_ID.fetch_add(
                1, ::std::sync::atomic::Ordering::Relaxed)
        }
    }
}

impl ::std::cmp::PartialEq for ShapeBase {
    fn eq(&self, other: &ShapeBase) -> bool {
        self.object2world == other.object2world
            && self.world2object == other.world2object
            && self.reverse_orientation == other.reverse_orientation
            && self.transform_swaps_handedness == other.transform_swaps_handedness
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ShapeIntersection<'a> {
    pub t_hit: f32,
    pub ray_epsilon: f32,
    pub dg: DifferentialGeometry<'a>
}

impl<'a> ShapeIntersection<'a> {
    pub fn new(t: f32, eps: f32, dgeom: DifferentialGeometry<'a>)
           -> ShapeIntersection<'a> {
        ShapeIntersection {
            t_hit: t,
            ray_epsilon: eps,
            dg: dgeom
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape<'a> {
    Sphere(Sphere),
    Disk(Disk),
    Cylinder(Cylinder),
    Triangle(Triangle<'a>),
    TriangleMesh(Mesh),
    LoopSubdiv(LoopSubdiv)
}

impl<'a> HasBounds for Shape<'a> {
    fn world_bound(&self) -> BBox {
        match self {
            &Shape::Sphere(ref s) => s.world_bound(),
            &Shape::Disk(ref d) => d.world_bound(),
            &Shape::Cylinder(ref c) => c.world_bound(),
            &Shape::Triangle(ref t) => t.world_bound(),
            &Shape::TriangleMesh(ref m) => m.world_bound(),
            &Shape::LoopSubdiv(ref m) => m.world_bound()
        }
    }
}

impl<'a> Refinable<'a> for Shape<'a> {
    // Default is all shapes can intersect..
    fn is_refined(&'a self) -> bool {
        match self {
            &Shape::Sphere(_) => true,
            &Shape::Disk(_) => true,
            &Shape::Cylinder(_) => true,
            &Shape::Triangle(_) => true,
            &Shape::TriangleMesh(_) => false,
            &Shape::LoopSubdiv(_) => false
        }
    }

    fn refine(&'a self) -> Vec<Shape<'a>> {
        match self {
            &Shape::Sphere(ref s) => vec![Shape::Sphere(s.clone())],
            &Shape::Disk(ref d) => vec![Shape::Disk(d.clone())],
            &Shape::Cylinder(ref c) => vec![Shape::Cylinder(c.clone())],
            &Shape::Triangle(ref t) => vec![Shape::Triangle(t.clone())],
            &Shape::TriangleMesh(ref m) =>
                m.refine().iter().cloned().map(Shape::Triangle).collect(),
            &Shape::LoopSubdiv(ref m) => m.refine().iter().cloned().map(Shape::TriangleMesh).collect()
        }
    }
}

impl<'a> Shape<'a> {
    pub fn base(&'a self) -> &'a ShapeBase {
        match self {
            &Shape::Sphere(ref s) => &s.base(),
            &Shape::Disk(ref d) => &d.base(),
            &Shape::Cylinder(ref c) => &c.base(),
            &Shape::Triangle(ref t) => &t.base(),
            &Shape::TriangleMesh(ref m) => &m.base(),
            &Shape::LoopSubdiv(ref m) => &m.base()
        }
    }

    pub fn sphere(o2w: Transform, w2o: Transform, ro: bool,
                  rad: f32, z0: f32, z1: f32, pm: f32) -> Shape<'a> {
        Shape::Sphere( Sphere::new(o2w, w2o, ro, rad, z0, z1, pm) )
    }

    pub fn cylinder(o2w: Transform, w2o: Transform, ro: bool,
                    rad: f32, z0: f32, z1: f32, pm: f32) -> Shape<'a> {
        Shape::Cylinder( Cylinder::new(o2w, w2o, ro, rad, z0, z1, pm) )
    }

    pub fn disk(o2w: Transform, w2o: Transform, ro: bool,
                ht: f32, r: f32, ri: f32, t_max: f32) -> Shape<'a> {
        Shape::Disk( Disk::new(o2w, w2o, ro, ht, r, ri, t_max) )
    }

    pub fn triangle_mesh(o2w: Transform, w2o: Transform, ro: bool, vi: &[usize],
                         _p: &[Point], _n: Option<&[Normal]>, _s: Option<&[Vector]>,
                         uv: Option<&[f32]>, _atex: Option<Rc<Texture<f32>>>) -> Shape<'a> {
        Shape::TriangleMesh( Mesh::new(o2w, w2o, ro, vi, _p, _n, _s, uv, _atex) )
    }

    pub fn loop_subdiv(o2w: Transform, w2o: Transform, ro: bool,
                       vertex_indices: &[usize], points: &[Point], nl: usize) -> Shape<'a> {
        Shape::LoopSubdiv( LoopSubdiv::new(o2w, w2o, ro, vertex_indices, points, nl) )
    }

    pub fn object_bound(&'a self) -> BBox {
        match self {
            &Shape::Sphere(ref s) => s.object_bound(),
            &Shape::Disk(ref d) => d.object_bound(),
            &Shape::Cylinder(ref c) => c.object_bound(),
            &Shape::Triangle(ref t) => t.object_bound(),
            &Shape::TriangleMesh(ref m) => m.object_bound(),
            &Shape::LoopSubdiv(ref m) => m.object_bound()
        }
    }

    pub fn get_shading_geometry(&'a self, o2w: &Transform,
                            dg: DifferentialGeometry<'a>)
                            -> DifferentialGeometry<'a> {
        match self {
            &Shape::Triangle(ref t) => t.get_shading_geometry(o2w, dg),
            _ => dg
        }
    }

    pub fn area(&self) -> f32 {
        match self {
            &Shape::Sphere(ref s) => s.area(),
            &Shape::Disk(ref d) => d.area(),
            &Shape::Cylinder(ref c) => c.area(),
            &Shape::Triangle(ref t) => t.area(),
            _ => self.refine().iter().fold(0f32, |a, t| a + t.area())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use transform::transform::Transform;

    #[test]
    fn it_can_be_created() {
        let some_shape = ShapeBase::new(Transform::new(), Transform::new(), false);
        assert!(some_shape.shape_id >= 0);
        assert_eq!(ShapeBase::new(Transform::new(), Transform::new(), false),
                   ShapeBase {
                       object2world: Transform::new(),
                       world2object: Transform::new(),
                       reverse_orientation: false,
                       transform_swaps_handedness: false,
                       shape_id: some_shape.shape_id + 1
                   });
    }

    #[test]
    fn two_shapes_can_be_equal() {
        assert_eq!(ShapeBase::new(Transform::new(), Transform::new(), false),
                   ShapeBase::new(Transform::new(), Transform::new(), false));
    }
}
