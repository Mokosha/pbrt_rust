use std::borrow::Borrow;
use std::sync::{Arc, Weak};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use geometry::normal::Normal;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Vector;
use primitive::Refinable;
use shape::mesh::Mesh;
use shape::shape::ShapeBase;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

fn next(i: usize) -> usize { (i + 1) % 3 }
fn prev(i: usize) -> usize { (i + 2) % 3 }

fn beta(valence: usize) -> f32 {
    if valence == 3 {
        3f32 / 16f32
    } else {
        3f32 / ((8 * valence) as f32)
    }
}

// !SPEED! This is a really poor approximation to the C++ code that was provided
// in the book. Right now we have vectors of reference counted vertices and faces.
// This means that we need to allocate a new vertex/face every time we add one to
// the array. It would be significantly faster (and increase cache coherence) if
// we had something like a RcVec that lets us create a whole reference counted
// vector and then give out Weak references to the internal elements.

#[derive(Debug, Clone)]
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

    fn fold_faces<F, T>(&self, start: T, closure: F)
                        -> T where F: Fn(T, &SDFace) -> T {
        let sf = match self.start_face.clone() {
            None => return start,
            Some(_f) => _f.upgrade().unwrap()
        };

        if self.boundary {
            // Compute valence of boundary vertex
            let mut first_face : Arc<SDFace> = sf;
            let mut f : Option<Arc<SDFace>> = first_face.prev_face(self);

            while f != None {
                let face : Arc<SDFace> = f.unwrap();
                first_face = face.clone();
                f = face.prev_face(self);
            }

            f = Some(first_face);
            let mut t = start;

            while f != None {
                let face = f.unwrap();
                t = closure(t, face.as_ref());
                f = face.next_face(self);
            }

            t
        } else {
            // Compute valence of interior vertex
            let mut t = closure(start, sf.as_ref());
            let mut f = sf.next_face(self).unwrap();
            while f != sf {
                t = closure(t, f.as_ref());
                f = f.next_face(self).unwrap();
            }

            t
        }
    }

    fn valence(&self) -> usize {
        self.fold_faces(1, |nf, _| { nf + 1 })
    }

    // !SPEED! Ideally we shouldn't be allocating a vec here -- better
    // to figure out how to do something like alloca in C99...
    fn one_ring(&self) -> Vec<Point> {
        self.fold_faces(Vec::new(), |ps, nf| {
            let mut v = ps;
            if v.len() == 0 {
                v.push(nf.prev_vert(self).upgrade().unwrap().p.clone());
                v.push(nf.next_vert(self).upgrade().unwrap().p.clone());
            } else {
                v.push(nf.next_vert(self).upgrade().unwrap().p.clone());
            }
            v
        })
    }

    fn weight_boundary(&self, beta: f32) -> Point {
        let p_ring = self.one_ring();
        assert!(p_ring.len() > 0);
        (1.0 - 2.0 * beta) * &self.p + beta * &p_ring[0] + beta * &p_ring[p_ring.len() - 1]
    }

    fn weight_one_ring(&self, beta: f32) -> Point {
        let p_ring = self.one_ring();
        let mut p = (1.0 - (p_ring.len() as f32) * beta) * &self.p;
        for vi in p_ring.iter() {
            p = p + beta * vi;
        }
        p
    }
}

#[derive(Debug)]
struct SDFace {
    v: [Weak<SDVertex>; 3],
    f: [Option<Weak<SDFace>>; 3],
    children: [Option<Weak<SDFace>>; 4],
}

impl ::std::clone::Clone for SDFace {
    fn clone(&self) -> SDFace {
        SDFace {
            v: [self.v[0].clone(), self.v[1].clone(), self.v[2].clone()],
            f: [self.f[0].clone(), self.f[1].clone(), self.f[2].clone()],
            children: [
                self.children[0].clone(),
                self.children[1].clone(),
                self.children[2].clone(),
                self.children[3].clone()]
        }
    }
}

impl ::std::cmp::PartialEq for SDFace {
    fn eq(&self, other: &SDFace) -> bool {
        self.v.iter().zip(other.v.iter()).fold(true, |r, (p, q)| {
            r && p.upgrade().unwrap() == q.upgrade().unwrap()
        })
    }
}

impl SDFace {
    fn new(v1: Weak<SDVertex>, v2: Weak<SDVertex>, v3: Weak<SDVertex>) -> SDFace {
        SDFace {
            v: [v1.clone(), v2.clone(), v3.clone()],
            f: [None, None, None],
            children: [None, None, None, None]
        }
    }

    fn vnum(&self, v: &SDVertex) -> usize {
        for i in 0..3 {
            if (*self.v[i].upgrade().unwrap()).borrow() == v {
                return i;
            }
        }

        panic!("Basic logic error in SDFace::vnum()");
    }

    fn next_face(&self, v: &SDVertex) -> Option<Arc<SDFace>> {
        self.f[self.vnum(v)].clone().map(|fr| fr.upgrade().unwrap())
    }

    fn prev_face(&self, v: &SDVertex) -> Option<Arc<SDFace>> {
        self.f[prev(self.vnum(v))].clone().map(|fr| fr.upgrade().unwrap())
    }

    fn next_vert(&self, v: &SDVertex) -> Weak<SDVertex> {
        self.v[next(self.vnum(v))].clone()
    }

    fn prev_vert(&self, v: &SDVertex) -> Weak<SDVertex> {
        self.v[prev(self.vnum(v))].clone()
    }

    fn other_vert(&self, v1: &SDVertex, v2: &SDVertex) -> Weak<SDVertex> {
        for vtx in self.v.iter() {
            let real_vtx = vtx.upgrade().unwrap();
            if real_vtx.as_ref() != v1 && real_vtx.as_ref() != v2 {
                return vtx.clone();
            }
        }

        panic!("Basic logic error in SDFace::other_vert()");
    }
}

#[derive(Debug)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct LoopSubdiv {
    base: ShapeBase,
    n_levels: usize,
    vertices: Vec<Arc<SDVertex>>,
    faces: Vec<Arc<SDFace>>,
    max_vert_id: usize
}

impl LoopSubdiv {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool,
               vertex_indices: &[usize], points: &[Point], nl: usize)
               -> LoopSubdiv {
        // Allocate vertices
        let mut vert_id = 0;
        let mut verts = {
            let mut vs = Vec::new();

            for p in points {
                vs.push(Arc::new(SDVertex::new(vert_id, p)));
                vert_id += 1;
            }

            vs
        };

        // Allocate faces
        debug_assert_eq!((vertex_indices.len() % 3), 0);
        let num_faces = vertex_indices.len() / 3;
        let mut faces: Vec<Arc<SDFace>> = {
            let mut vert_idxs = vertex_indices.iter();

            (0..num_faces).map(|_| {
                let v0 = *vert_idxs.next().unwrap();
                let v1 = *vert_idxs.next().unwrap();
                let v2 = *vert_idxs.next().unwrap();
                let f = Arc::new(SDFace::new(
                    Arc::downgrade(&verts[v0]),
                    Arc::downgrade(&verts[v1]),
                    Arc::downgrade(&verts[v2])));

                Arc::get_mut(&mut verts[v0]).unwrap().start_face = Some(Arc::downgrade(&f));
                Arc::get_mut(&mut verts[v1]).unwrap().start_face = Some(Arc::downgrade(&f));
                Arc::get_mut(&mut verts[v2]).unwrap().start_face = Some(Arc::downgrade(&f));

                f
            }).collect()
        };

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
                        Arc::get_mut(&mut e.f[0].as_mut().unwrap().upgrade().unwrap())
                            .unwrap().f[e.f0_edge_num] = Some(Arc::downgrade(f));
                        Arc::get_mut(f).as_mut().unwrap().f[edge_num] = e.f[0].clone();
                    }
                    edges.remove(&key);
                } else {
                    // Handle new edge
                    let mut e = SDEdge::new(f.v[v0].clone(), f.v[v1].clone());
                    e.f[0] = Some(Arc::downgrade(f));
                    e.f0_edge_num = edge_num;
                    edges.insert(key, e);
                }
            }
        }

        // Finish vertex initialization
        for v in verts.iter_mut() {
            let boundary = {
                let sf = {
                    match v.start_face.clone() {
                        None => continue,
                        Some(_f) => _f.upgrade().unwrap()
                    }
                };

                let mut f = sf.clone();
                let mut is_boundary = false;
                loop {
                    f = match f.next_face((*v).borrow()) {
                        None => {
                            is_boundary = true;
                            break;
                        },

                        Some(_f) => _f
                    };

                    if f == sf { break };
                }

                is_boundary
            };

            Arc::get_mut(v).unwrap().boundary = boundary;
            Arc::get_mut(v).unwrap().regular =
                (!v.boundary && v.valence() == 6) || (v.boundary && v.valence() == 4);
        }

        LoopSubdiv {
            base: ShapeBase::new(o2w, w2o, ro),
            n_levels: nl,
            vertices: verts,
            faces: faces,
            max_vert_id: vert_id
        }
    }

    pub fn base<'a>(&'a self) -> &'a ShapeBase { &self.base }

    pub fn object_bound(&self) -> BBox {
        self.vertices.iter().fold(BBox::new(), |b, v| b.unioned_with_ref(&v.p))
    }
}

impl Refinable<Mesh> for LoopSubdiv {
    fn is_refined(&self) -> bool { false }
    fn refine(self) -> Vec<Mesh> {
        let mut f = self.faces.clone();
        let mut v = self.vertices.clone();
        
        let mut vtx_id = self.max_vert_id;
        for _ in 0..self.n_levels {
            // Update f and v for next level of subdivision
            let mut new_vertices = Vec::new();

            // Allocate next level of children in mesh tree
            for vtx in v.iter_mut() {
                // Determine new vertex position
                let p = if vtx.boundary {
                    // Apply boundary rule for even vertex
                    vtx.weight_boundary(1f32 / 8f32)
                } else {
                    // Apply one-ring rule for even vertex
                    if vtx.regular {
                        vtx.weight_one_ring(1f32 / 16f32)
                    } else {
                        vtx.weight_one_ring(beta(vtx.valence()))
                    }
                };

                let mut new_vtx = Arc::new(SDVertex::new(vtx_id, &p));
                vtx_id += 1;

                Arc::get_mut(&mut new_vtx).unwrap().boundary = vtx.boundary;
                Arc::get_mut(&mut new_vtx).unwrap().regular = vtx.regular;

                Arc::get_mut(vtx).unwrap().child = Some(Arc::downgrade(&new_vtx));
                new_vertices.push(new_vtx);
            }

            // Compute new odd edge vertices
            let mut edge_verts: HashMap<SDEdge, Weak<SDVertex>> = HashMap::new();
            for face in f.iter_mut() {
                for k in 0..3 {
                    // Compute odd vertex on k'th edge
                    let edge = SDEdge::new(face.v[k].clone(), face.v[next(k)].clone());
                    if !edge_verts.contains_key(&edge) {
                        // Apply edge rules to compute new vertex position
                        let boundary = match face.f[k] {
                            None => false,
                            Some(_) => true
                        };

                        let pos = {
                            let v1 = edge.v[0].upgrade().unwrap();
                            let v2 = edge.v[1].upgrade().unwrap();
                            if boundary {
                                0.5 * &v1.p + 0.5 * &v2.p
                            } else {
                                let v3 = face
                                    .other_vert(v1.as_ref(), v2.as_ref())
                                    .upgrade().unwrap();

                                let v4 =
                                    face.f[k].clone().unwrap()
                                    .upgrade().unwrap()
                                    .other_vert(v1.as_ref(), v2.as_ref())
                                    .upgrade().unwrap();

                                let mut p = (3f32 / 8f32) * &v1.p;
                                p = p + (3f32 / 8f32) * &v2.p;
                                p = p + (1f32 / 8f32) * &v3.p;
                                p = p + (1f32 / 8f32) * &v4.p;
                                p
                            }
                        };

                        // Create and initialize new odd vertex
                        let mut vert = Arc::new(SDVertex::new(vtx_id, &pos));
                        vtx_id += 1;

                        Arc::get_mut(&mut vert).unwrap().boundary = boundary;
                        Arc::get_mut(&mut vert).unwrap().regular = true;

                        edge_verts.insert(edge, Arc::downgrade(&vert));
                        new_vertices.push(vert);
                    }
                }
            }

            // Create child faces, set verts based on edges, and set
            // start face for intermediate edge verts to center face...
            let mut new_faces = Vec::new();
            for f in f.iter_mut() {
                let mut cvs : Vec<Weak<SDVertex>> = (0..3).map(|k| {
                    let edge = SDEdge::new(f.v[k].clone(), f.v[next(k)].clone());
                    assert!(edge_verts.contains_key(&edge));
                    edge_verts.get(&edge).unwrap().clone()
                }).collect();

                // Allocate new faces...
                let f1 = Arc::new(SDFace::new(f.v[0].clone(), cvs[0].clone(), cvs[2].clone()));
                let f2 = Arc::new(SDFace::new(cvs[0].clone(), f.v[1].clone(), cvs[1].clone()));
                let f3 = Arc::new(SDFace::new(cvs[2].clone(), cvs[1].clone(), f.v[2].clone()));
                let f4 = Arc::new(SDFace::new(cvs[0].clone(), cvs[1].clone(), cvs[2].clone()));

                // Set f4 as the start face for all of the child vertices...
                for cv in cvs.iter_mut() {
                    Arc::get_mut(&mut cv.upgrade().unwrap()).unwrap().start_face = Some(Arc::downgrade(&f4))
                }

                // Set f1-f4 as the children for this face...
                Arc::get_mut(f).unwrap().children = [
                    Some(Arc::downgrade(&f1)),
                    Some(Arc::downgrade(&f2)),
                    Some(Arc::downgrade(&f3)),
                    Some(Arc::downgrade(&f4))];

                // Add each face to the list of new faces..
                new_faces.push(f1);
                new_faces.push(f2);
                new_faces.push(f3);
                new_faces.push(f4);
            }

            // Update new mesh topology
            /* Update even vertex face pointers */
            for vert in v.iter_mut() {
                let vert_num = vert.start_face.clone().unwrap().upgrade().unwrap().vnum(vert);
                Arc::get_mut(&mut vert.child.clone().unwrap().upgrade().unwrap()).unwrap().start_face =
                    vert.start_face.clone().unwrap().upgrade().unwrap().children[vert_num].clone()
            }

            /* Update face neighbor pointers */
            for face in f.iter_mut() {
                for k in 0..3 {
                    // Update f pointers for siblings
                    Arc::get_mut(&mut face.children[3].clone().unwrap().upgrade().unwrap()).unwrap().f[k] =
                        face.children[next(k)].clone();
                    Arc::get_mut(&mut face.children[k].clone().unwrap().upgrade().unwrap()).unwrap().f[next(k)] =
                        face.children[3].clone();
                    
                    // Update children f pointers for neighbor children
                    let mut child_ref = face.children[k].clone().unwrap().upgrade().unwrap();
                    let mut f2 = face.f[k].clone();
                    if let Some(f2_wref) = f2 {
                        let f2_ref = f2_wref.upgrade().unwrap();
                        Arc::get_mut(&mut child_ref).unwrap().f[k] =
                            f2_ref.children[f2_ref.vnum(face.v[k].upgrade().unwrap().as_ref())].clone();
                    } else {
                        Arc::get_mut(&mut child_ref).unwrap().f[k] = None;
                    }

                    f2 = face.f[prev(k)].clone();
                    if let Some(f2_wref) = f2 {
                        let f2_ref = f2_wref.upgrade().unwrap();
                        Arc::get_mut(&mut child_ref).unwrap().f[prev(k)] =
                            f2_ref.children[f2_ref.vnum(face.v[k].upgrade().unwrap().as_ref())].clone();
                    } else {
                        Arc::get_mut(&mut child_ref).unwrap().f[prev(k)] = None;
                    }
                }
            }

            // Prepare for next level of subdivision
            f = new_faces;
            v = new_vertices;
        }

        // Push vertices to limit surface
        let p_limit : Vec<_> = v.iter().map(|vert| {
            if vert.boundary {
                vert.weight_boundary(1f32 / 5f32)
            } else {
                let valence = vert.valence();
                let gamma = 1.0 / ((valence as f32) + 3.0 / (8.0 * beta(valence)));
                vert.weight_one_ring(gamma)
            }
        }).collect();

        for (k, vert) in v.iter_mut().enumerate() {
            Arc::get_mut(vert).unwrap().p = p_limit[k].clone();
        }

        // Compute vertex tangents on limit surface
        let ns : Vec<_> = v.iter().map(|vert| {
            let mut s = Vector::new();
            let mut t = Vector::new();

            let valence = vert.valence();

            let p_ring = vert.one_ring();
            if vert.boundary {
                // Compute tangents of interior vertex
                for (k, p) in p_ring.iter().enumerate() {
                    let factor = 2f32 * ::std::f32::consts::PI * (k as f32) / (valence as f32);
                    s = s + factor.cos() * Vector::from(p.clone());
                    t = t + factor.sin() * Vector::from(p.clone());
                }
            } else {
                // Compute tangents of boundary vertex
                s = &p_ring[valence - 1] - &p_ring[0];
                t = match valence {
                    2 => Vector::from(&p_ring[0] + &p_ring[1] - 2.0 * &vert.p),
                    3 => &p_ring[1] - &vert.p,
                    4 => Vector::from(-1.0 * &p_ring[0] + 2.0 * &p_ring[1] + 2.0 * &p_ring[2] +
                                      -1.0 * &p_ring[3] - 2.0 * &vert.p),
                    _ => {
                        let theta = ::std::f32::consts::PI / ((valence - 1) as f32);
                        let mut r = Vector::from(theta.sin() * (&p_ring[0] + &p_ring[valence - 1]));
                        for (k, p) in p_ring.iter().enumerate() {
                            let wt = (2.0 * theta.cos() - 2.0) * ((k as f32) * theta).sin();
                            r = r + Vector::from(wt * p);
                        }
                        -r
                    }
                }
            }

            Normal::from(s.cross(&t).normalize())
        }).collect();

        // Create TriangleMesh from subdivision mesh
        let mut used_verts: HashMap<Arc<SDVertex>, usize> = HashMap::new();
        let mut used_vert_id = 0;
        for vert in v.iter() {
            used_verts.insert(vert.clone(), used_vert_id);
            used_vert_id = used_vert_id + 1;
        }

        let mut indices = Vec::with_capacity(f.len() * 3);
        for face in f.iter() {
            for j in 0..3 {
                indices.push(*(used_verts.get(&face.v[j].upgrade().unwrap()).unwrap()));
            }
        }

        vec![Mesh::new(self.base.object2world.clone(),
                       self.base.world2object.clone(),
                       self.base.reverse_orientation,
                       &*indices.into_boxed_slice(),
                       &*p_limit.into_boxed_slice(),
                       Some(&*ns.into_boxed_slice()),
                       None, None, None)]
    }
}

impl HasBounds for LoopSubdiv {
    fn world_bound(&self) -> BBox {
        let o2w = &self.base().object2world;
        self.vertices.iter().fold(BBox::new(), |b, v| b.unioned_with(o2w.t(&v.p)))
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// !FIXME! We have to wait for https://github.com/rust-lang/rust/issues/30658
// to be fixed before we can implement these fixes...

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
                                     &TET_TRIS, &TET_PTS, 1);
        assert_eq!(subdiv.n_levels, 1);
        assert_eq!(subdiv.vertices.len(), 4);
        assert_eq!(subdiv.faces.len(), 4);
    }

    #[ignore]
    #[test]
    fn it_has_object_space_bounds() {
        unimplemented!();
    }

    #[ignore]
    #[test]
    fn it_has_world_space_bounds() {
        unimplemented!();
    }

    #[ignore]
    #[test]
    fn it_can_be_refined() {
        unimplemented!();
    }
}
