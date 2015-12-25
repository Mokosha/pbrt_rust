use std::ops::{Add, Rem};
use std::num::{One};

use geometry::point::Point;
use shape::shape::IsShape;
use shape::shape::Shape;
use transform::transform::Transform;

fn next(i: usize) -> usize { (i + 1) % 3 }
fn prev(i: usize) -> usize { (i + 2) % 3 }

struct SDVertex<'a> {
    p: Point,
    start_face: Option<&'a SDFace<'a>>,
    child: Option<&'a SDVertex<'a>>,
    regular: bool,
    boundary: bool
}

impl<'a> SDVertex<'a> {
    fn new(_p: &Point) -> SDVertex<'a> {
        SDVertex {
            p: _p.clone(),
            start_face: None,
            child: None,
            regular: false,
            boundary: false
        }
    }
}

struct SDFace<'a> {
    v: [Option<&'a SDVertex<'a>>; 3],
    f: [Option<&'a SDFace<'a>>; 3],
    children: [Option<&'a SDFace<'a>>; 4],
}

impl<'a> SDFace<'a> {
    fn new() -> SDFace<'a> {
        SDFace {
            v: [None, None, None],
            f: [None, None, None],
            children: [None, None, None, None]
        }
    }
}

struct SDEdge<'a> {
    v: [&'a SDVertex<'a>; 2],
    f: [&'a SDFace<'a>; 2],
    f0_edge_num: usize
}

pub struct LoopSubdiv<'a> {
    shape: Shape,
    n_levels: usize,
    vertices: Vec<SDVertex<'a>>,
    faces: Vec<SDFace<'a>>,
}

impl<'a> LoopSubdiv<'a> {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool, num_faces: usize,
               vertex_indices: &[usize], points: &[Point], nl: usize)
               -> LoopSubdiv<'a> {
        // Allocate vertices and faces
        let mut verts = Vec::new();
        for p in points {
            verts.push(SDVertex::new(p));
        }

        let mut faces = Vec::with_capacity(num_faces);
        for _ in 0..num_faces {
            faces.push(SDFace::new());
        }

        // Set face to vertex pointers
        // Set neighbor pointers in faces
        // Finish vertex initialization
        LoopSubdiv {
            shape: Shape::new(o2w, w2o, ro),
            n_levels: nl,
            vertices: verts,
            faces: faces
        }
    }
}
