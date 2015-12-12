use geometry::normal::Normal;
use geometry::point::Point;
use geometry::vector::Vector;
use shape::shape::Shape;
use texture::Texture;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

#[derive(Clone, Debug, PartialEq)]
struct Mesh {
    shape: Shape,
    vertex_index: Vec<usize>,
    p: Vec<Point>,
    n: Option<Vec<Normal>>,
    s: Option<Vec<Vector>>,
    uvs: Option<Vec<f32>>,
    atex: ::std::rc::Rc<Texture<f32>>
}

impl Mesh {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool, vi: &[usize],
               _p: &[Point], _n: Option<&[Normal]>, _s: Option<&[Vector]>,
               uv: Option<&[f32]>, _atex: ::std::rc::Rc<Texture<f32>>) -> Mesh {
        let xf = o2w.clone();
        Mesh {
            shape: Shape::new(o2w, w2o, ro),
            vertex_index: vi.to_vec(),
            p: _p.iter().map(|x| xf.t(x)).collect(),
            n: _n.map(|v| v.to_vec()),
            s: _s.map(|v| v.to_vec()),
            uvs: uv.map(|v| v.to_vec()),
            atex: _atex.clone()
        }
    }
}
