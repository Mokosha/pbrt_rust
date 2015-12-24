use geometry::point::Point;

struct SDVertex<'a> {
    p: Point,
    start_face: Option<&'a SDFace<'a>>,
    child: Option<&'a SDVertex<'a>>,
    regular: bool,
    boundary: bool
}

struct SDFace<'a> {
    v: [Option<&'a SDVertex<'a>>; 3],
    f: [Option<&'a SDFace<'a>>; 3],
    children: [Option<&'a SDFace<'a>>; 4],
}

struct SDEdge<'a> {
    v: [&'a SDVertex<'a>; 2],
    f: [&'a SDFace<'a>; 2],
    f0_edge_num: usize
}

pub struct LoopSubdiv<'a> {
    n_levels: usize,
    vertices: Vec<SDVertex<'a>>,
    faces: Vec<SDFace<'a>>,
}
