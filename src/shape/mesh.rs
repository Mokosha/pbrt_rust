use bbox::BBox;
use bbox::Union;
use geometry::normal::Normal;
use geometry::point::Point;
use geometry::vector::Vector;
use shape::shape::IsShape;
use shape::shape::Shape;
use texture::Texture;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
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

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::point::Point;
    use transform::transform::Transform;

    use texture::white_float_tex;

    // Tetrahedron
    static tet_pts : [Point; 4] =
        [Point { x: 0.0, y: 0.0, z: 0.0 },
         Point { x: 1.0, y: 0.0, z: 0.0 },
         Point { x: 0.0, y: 1.0, z: 0.0 },
         Point { x: 0.0, y: 0.0, z: 1.0 }];
    static tet_tris : [usize; 12] =
        [ 0, 3, 2, 0, 1, 2, 0, 3, 1, 1, 2, 3 ];

    #[test]
    fn it_can_be_created() {
        let mesh = Mesh::new(Transform::new(), Transform::new(), false,
                             &tet_tris, &tet_pts, None, None, None,
                             ::std::rc::Rc::new(white_float_tex()));
        // Make sure that all of the indices and points remained untransformed...
        assert_eq!(mesh.vertex_index, tet_tris.to_vec());
        assert_eq!(mesh.p, tet_pts.to_vec());

        // If we rotate it about y by 90 degrees then it should be OK as well
        let xf = Transform::rotate_y(90.0);
        let mesh2 = Mesh::new(xf.clone(), xf.inverse(), false,
                              &tet_tris, &tet_pts, None, None, None,
                              ::std::rc::Rc::new(white_float_tex()));

        assert_eq!(mesh2.vertex_index, tet_tris.to_vec());
        assert!(mesh2.p.iter().zip(vec![
            Point::new_with(0.0, 0.0, 0.0),
            Point::new_with(0.0, 0.0, -1.0),
            Point::new_with(0.0, 1.0, 0.0),
            Point::new_with(1.0, 0.0, 0.0)]).any(
            |(p, q)| {
                (p - q).length_squared() < 1e-6
            }));
    }
}
