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

impl IsShape for Mesh {
    fn get_shape<'a>(&'a self) -> &'a Shape { &(self.shape) }
    fn object_bound(&self) -> BBox {
        let w2o = &(self.get_shape().world2object);
        self.p.iter().fold(BBox::new(), |b, p| b.unioned_with(&(w2o.t(p))))
    }

    fn world_bound(&self) -> BBox {
        self.p.iter().fold(BBox::new(), |b, p| b.unioned_with(p))
    }

    // Cannot intersect meshes directly.
    fn can_intersect(&self) -> bool { false }

}

#[cfg(test)]
mod tests {
    use super::*;
    use bbox::BBox;
    use geometry::point::Point;
    use shape::shape::IsShape;
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

    #[test]
    fn it_has_object_space_bounds() {
        let xf = Transform::rotate_y(90.0);
        let mesh = Mesh::new(xf.clone(), xf.inverse(), false,
                             &tet_tris, &tet_pts, None, None, None,
                             ::std::rc::Rc::new(white_float_tex()));
        let expected = BBox::new_with(Point::new(),
                                      Point::new_with(1.0, 1.0, 1.0));

        assert_eq!(mesh.object_bound(), expected);
        assert_eq!(Mesh::new(Transform::new(), Transform::new(), false,
                             &tet_tris, &tet_pts, None, None, None,
                             ::std::rc::Rc::new(white_float_tex())).object_bound(),
                   expected);
    }

    #[test]
    fn it_has_world_space_bounds() {
        let xf = Transform::rotate_y(90.0);
        let mesh = Mesh::new(xf.clone(), xf.inverse(), false,
                             &tet_tris, &tet_pts, None, None, None,
                             ::std::rc::Rc::new(white_float_tex()));

        let expected = BBox::new_with(Point::new_with(0.0, 0.0, -1.0),
                                      Point::new_with(1.0, 1.0, 0.0));


        assert!((mesh.world_bound().p_min - expected.p_min).length_squared() < 1e-6);
        assert!((mesh.world_bound().p_max - expected.p_max).length_squared() < 1e-6);
        assert_eq!(Mesh::new(Transform::new(), Transform::new(), false,
                             &tet_tris, &tet_pts, None, None, None,
                             ::std::rc::Rc::new(white_float_tex())).world_bound(),
                   BBox::new_with(Point::new(), Point::new_with(1.0, 1.0, 1.0)));
    }
}
