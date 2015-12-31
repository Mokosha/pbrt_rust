use std::rc::{Rc, Weak};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

use geometry::point::Point;
use shape::shape::IsShape;
use shape::shape::Shape;
use transform::transform::Transform;

fn next(i: usize) -> usize { (i + 1) % 3 }
fn prev(i: usize) -> usize { (i + 2) % 3 }

// !SPEED! This is a really poor approximation to the C++ code that was provided
// in the book. Right now we have vectors of reference counted vertices and faces.
// This means that we need to allocate a new vertex/face every time we add one to
// the array. It would be significantly faster (and increase cache coherence) if
// we had something like a RcVec that lets us create a whole reference counted
// vector and then give out Weak references to the internal elements.

struct SDVertex {
    p: Point,
    id: usize,  // We need this for ordering edges properly
    start_face: Option<Weak<SDFace>>,
    child: Option<Weak<SDVertex>>,
    regular: bool,
    boundary: bool
}

impl ::std::cmp::PartialEq for SDVertex {
    fn eq(&self, other: &SDVertex) -> bool {
        return self.id.eq(&other.id);
    }
}

impl ::std::cmp::Eq for SDVertex { }

impl Hash for SDVertex {
    fn hash<H>(&self, state: &mut H) where H : Hasher {
        self.id.hash(state);
        self.regular.hash(state);
        self.boundary.hash(state);
    }
}

impl SDVertex {
    fn new(_id: usize, _p: &Point) -> SDVertex {
        SDVertex {
            p: _p.clone(),
            id: _id,
            start_face: None,
            child: None,
            regular: false,
            boundary: false
        }
    }
}

pub struct SDFace {
    v: [Weak<SDVertex>; 3],
    f: [Option<Weak<SDFace>>; 3],
    children: [Option<Weak<SDFace>>; 4],
}

impl SDFace {
    fn new(v1: Weak<SDVertex>, v2: Weak<SDVertex>, v3: Weak<SDVertex>) -> SDFace {
        SDFace {
            v: [v1.clone(), v2.clone(), v3.clone()],
            f: [None, None, None],
            children: [None, None, None, None]
        }
    }
}

struct SDEdge {
    v: [Weak<SDVertex>; 2],
    f: [Option<Weak<SDFace>>; 2],
    f0_edge_num: usize
}

impl ::std::cmp::PartialEq for SDEdge {
    fn eq(&self, other: &SDEdge) -> bool {
        self.v[0].upgrade() == other.v[0].upgrade() &&
        self.v[1].upgrade() == other.v[1].upgrade()
    }
}

impl ::std::cmp::Eq for SDEdge { }

impl Hash for SDEdge {
    fn hash<H>(&self, state: &mut H) where H : Hasher {
        self.v[0].upgrade().unwrap().hash(state);
        self.v[1].upgrade().unwrap().hash(state);
    }
}

impl SDEdge {
    fn new(v1: Weak<SDVertex>, v2: Weak<SDVertex>) -> SDEdge {
        let (min_v, max_v) =
            if v1.upgrade().unwrap().id < v2.upgrade().unwrap().id {
                (v1.clone(), v2.clone())
            } else {
                (v2.clone(), v1.clone())
            };

        SDEdge {
            v: [min_v, max_v],
            f: [None, None],
            f0_edge_num: 4
        }
    }
}

pub struct LoopSubdiv {
    shape: Shape,
    n_levels: usize,
    vertices: Vec<Rc<SDVertex>>,
    faces: Vec<Rc<SDFace>>,
    max_vert_id: usize
}

impl LoopSubdiv {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool,
               num_faces: usize,
               vertex_indices: &[usize], points: &[Point], nl: usize)
               -> LoopSubdiv {
        debug_assert_eq!((vertex_indices.len() % 3), 0);

        // Allocate vertices and faces
        let mut vert_id = 0;
        let mut verts = {
            let mut vs = Vec::new();

            for p in points {
                vs.push(Rc::new(SDVertex::new(vert_id, p)));
                vert_id += 1;
            }

            vs
        };

        let mut faces: Vec<Rc<SDFace>> = {
            let mut vert_idxs = vertex_indices.iter();

            (0..num_faces).map(|_| {
                let mut v0 = verts[*vert_idxs.next().unwrap()].clone();
                let mut v1 = verts[*vert_idxs.next().unwrap()].clone();
                let mut v2 = verts[*vert_idxs.next().unwrap()].clone();
                Rc::new(SDFace::new(
                    Rc::downgrade(&v0),
                    Rc::downgrade(&v1),
                    Rc::downgrade(&v2)))
            }).collect()
        };

        // Set face to vertex pointers
        for f in faces.iter_mut() {
            for i in 0..3 {
                Rc::get_mut(&mut f.v[i].upgrade().unwrap()).unwrap().start_face = Some(Rc::downgrade(&f));
            }
        }

        // Set neighbor pointers in faces
        let mut edges: HashMap<(usize, usize), SDEdge> = HashMap::new();
        for f in faces.iter_mut() {
            for edge_num in 0..3 {
                let v0 = edge_num;
                let v1 = next(edge_num);

                // Update neighbor pointer for edge_num
                let key = {
                    let id1 = f.v[v0].clone().upgrade().unwrap().id;
                    let id2 = f.v[v1].clone().upgrade().unwrap().id;
                    if id1 < id2 { (id1, id2) } else { (id2, id1) }
                };

                if edges.contains_key(&key) {
                    {
                        let e = edges.get_mut(&key).unwrap();
                        assert!(e.f0_edge_num < 4);

                        // Handle previously seen edge
                        Rc::get_mut(&mut e.f[0].as_mut().unwrap().upgrade().unwrap())
                            .unwrap().f[e.f0_edge_num] = Some(Rc::downgrade(f));
                        Rc::get_mut(f).as_mut().unwrap().f[edge_num] = e.f[0].clone();
                    }
                    edges.remove(&key);
                } else {
                    // Handle new edge
                    let mut e = SDEdge::new(f.v[v0].clone(), f.v[v1].clone());
                    e.f[0] = Some(Rc::downgrade(f));
                    e.f0_edge_num = edge_num;
                    edges.insert(key, e);
                }
            }
        }

        // Finish vertex initialization
        LoopSubdiv {
            shape: Shape::new(o2w, w2o, ro),
            n_levels: nl,
            vertices: verts,
            faces: faces,
            max_vert_id: vert_id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use geometry::point::Point;
    use transform::transform::Transform;

    // Tetrahedron
    static TET_PTS : [Point; 4] =
        [Point { x: 0.0, y: 0.0, z: 0.0 },
         Point { x: 1.0, y: 0.0, z: 0.0 },
         Point { x: 0.0, y: 1.0, z: 0.0 },
         Point { x: 0.0, y: 0.0, z: 1.0 }];
    static TET_TRIS : [usize; 12] =
        [ 0, 3, 2, 0, 1, 2, 0, 3, 1, 1, 2, 3 ];

    #[ignore]
    #[test]
    fn it_can_be_created() {
        let subdiv = LoopSubdiv::new(Transform::new(), Transform::new(), false,
                                     4, &TET_TRIS, &TET_PTS, 1);
        assert_eq!(subdiv.n_levels, 1);
        assert_eq!(subdiv.vertices.len(), 4);
        assert_eq!(subdiv.faces.len(), 4);
    }
}
