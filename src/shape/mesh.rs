use std::rc::Rc;
use std::convert::AsRef;

use bbox::BBox;
use bbox::Union;
use diff_geom::DifferentialGeometry;
use geometry::normal::Normal;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Dot;
use geometry::vector::Vector;
use ray::Ray;
use shape::shape::IsShape;
use shape::shape::Shape;
use shape::shape::ShapeIntersection;
use texture::Texture;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

use geometry::vector::coordinate_system;

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    shape: Shape,
    vertex_index: Vec<usize>,
    p: Vec<Point>,
    n: Option<Vec<Normal>>,
    s: Option<Vec<Vector>>,
    uvs: Option<Vec<f32>>,
    atex: Option<Rc<Texture<f32>>>
}

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle<'a> {
    mesh: &'a Mesh,
    v: [usize; 3]
}

impl<'a> Triangle<'a> {
    pub fn get_vertices(&self) -> (&'a Point, &'a Point, &'a Point) {
        let p1 = &(self.mesh.p[self.v[0]]);
        let p2 = &(self.mesh.p[self.v[1]]);
        let p3 = &(self.mesh.p[self.v[2]]);

        (p1, p2, p3)
    }

    pub fn get_intersection_point(&self, r: &Ray) -> Option<(f32, f32, f32)> {
        // Compute s1
        let (p1, p2, p3) = self.get_vertices();

        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let s1 = r.d.clone().cross(&e2);
        let divisor = s1.dot(&e1);
        if divisor == 0f32 {
            return None;
        }

        // Compute first barycentric coordinate
        let inv_divisor = 1.0 / divisor;
        let s = &(r.o) - p1;
        let b1 = s1.dot(&s) * inv_divisor;
        if b1 < 0.0 || b1 > 1.0 {
            return None;
        }

        // Compute second barycentric coordinate
        let s2 = s.clone().cross(&e1);
        let b2 = r.d.dot(&s2) * inv_divisor;
        if b2 < 0.0 || (b1 + b2) > 1.0 {
            return None;
        }

        // Compute t to intersection point
        let t = e2.dot(&s2) * inv_divisor;

        if t < r.mint || t > r.maxt { None } else { Some((t, b1, b2)) }
    }

    pub fn get_uvs(&self) -> [[f32; 2]; 3] {
        if let Some(uvs) = self.mesh.uvs.as_ref() {
            [[uvs[2 * self.v[0]],
              uvs[2 * self.v[0] + 1]],
             [uvs[2 * self.v[1]],
              uvs[2 * self.v[1] + 1]],
             [uvs[2 * self.v[2]],
              uvs[2 * self.v[2] + 1]]]
        } else {
            [[0.0, 0.0],
             [1.0, 0.0],
             [1.0, 1.0]]
        }
    }
}

impl Mesh {
    pub fn new(o2w: Transform, w2o: Transform, ro: bool, vi: &[usize],
               _p: &[Point], _n: Option<&[Normal]>, _s: Option<&[Vector]>,
               uv: Option<&[f32]>, _atex: Option<Rc<Texture<f32>>>) -> Mesh {
        assert!(vi.len() % 3 == 0);
        let xf = o2w.clone();
        Mesh {
            shape: Shape::new(o2w, w2o, ro),
            vertex_index: vi.to_vec(),
            p: _p.iter().map(|x| xf.t(x)).collect(),
            n: _n.map(|v| v.to_vec()),
            s: _s.map(|v| v.to_vec()),
            uvs: uv.map(|v| v.to_vec()),
            atex: _atex.map(|t| t.clone())
        }
    }

    pub fn to_tris<'a>(&'a self) -> Vec<Triangle<'a>> {
        let mut indices = self.vertex_index.clone();
        let mut tris = Vec::new();
        while let (Some(v1), Some(v2), Some(v3)) =
            (indices.pop(), indices.pop(), indices.pop()) {
                tris.push(Triangle {
                    mesh: &self,
                    v: [v1, v2, v3]
                });
            }

        tris
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

impl<'a> IsShape for Triangle<'a> {
    fn get_shape<'b>(&'b self) -> &'b Shape { self.mesh.get_shape() }

    fn object_bound(&self) -> BBox {
        let (p1, p2, p3) = self.get_vertices();

        let w2o = &(self.get_shape().world2object);
        BBox::from(w2o.t(p1)).
            unioned_with(&(w2o.t(p2))).
            unioned_with(&(w2o.t(p3)))
    }

    fn world_bound(&self) -> BBox {
        let (p1, p2, p3) = self.get_vertices();

        BBox::from(p1.clone()).unioned_with(p2).unioned_with(p3)
    }

    fn intersect_p(&self, r: &Ray) -> bool {
        self.get_intersection_point(r).is_some()
    }

    fn intersect(&self, r: &Ray) -> Option<ShapeIntersection> {
        let (t, b1, b2) = {
            match self.get_intersection_point(r) {
                None => return None,
                Some(t) => t
            }
        };

        let (p1, p2, p3) = self.get_vertices();
        let uvs = self.get_uvs();

        // Compute deltas for triangle partial derivatives
        let du1 = uvs[0][0] - uvs[2][0];
        let du2 = uvs[1][0] - uvs[2][0];
        let dv1 = uvs[0][1] - uvs[2][1];
        let dv2 = uvs[1][1] - uvs[2][1];

        let dp1 = p1 - p3;
        let dp2 = p2 - p3;

        // Compute triangle partial derivatives
        let (dpdu, dpdv) = {
            let determinant = du1 * dv2 - dv1 * du2;
            if determinant == 0.0 {
                // Handle zero determinant for triangle partial
                // derivatives matrix
                coordinate_system(&((p3 - p1).cross(&(p2 - p1)).normalize()))
            } else {
                let inv_det = 1.0 / determinant;
                (( dv2 * &dp1 - dv1 * &dp2) * inv_det,
                 (-du2 * &dp1 + du1 * &dp2) * inv_det)
            }
        };

        // Interpolate (u, v) triangle parametric coordinates
        let b0 = 1.0 - b1 - b2;
        let tu = b0 * uvs[0][0] + b1 * uvs[1][0] + b2 * uvs[2][0];
        let tv = b0 * uvs[0][1] + b1 * uvs[1][1] + b2 * uvs[2][1];

        // Test intersection against alpha texture, if present
        let dg = DifferentialGeometry::new_with(
            r.point_at(t), dpdu, dpdv, Normal::new(), Normal::new(), tu, tv,
            Some(self.get_shape()));

        if let Some(tex_ref) = self.mesh.atex.as_ref().map(|t| t.clone()) {
            if (*tex_ref).evaluate(&dg) == 0.0 {
                return None
            }
        }

        Some(ShapeIntersection::new(t, t * 5e-4, dg))
    }
}

impl<'a> ::std::ops::Index<usize> for Triangle<'a> {
    type Output = Point;
    fn index(&self, i: usize) -> &Point {
        match i {
            0 ... 2 => &(self.mesh.p[self.v[i]]),
            _ => panic!("Error - Triangle index out of bounds!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bbox::BBox;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use ray::Ray;
    use shape::shape::IsShape;
    use transform::transform::Transform;

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
                             &tet_tris, &tet_pts, None, None, None, None);
        // Make sure that all of the indices and points remained untransformed...
        assert_eq!(mesh.vertex_index, tet_tris.to_vec());
        assert_eq!(mesh.p, tet_pts.to_vec());

        // If we rotate it about y by 90 degrees then it should be OK as well
        let xf = Transform::rotate_y(90.0);
        let mesh2 = Mesh::new(xf.clone(), xf.inverse(), false,
                              &tet_tris, &tet_pts, None, None, None, None);

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
                             &tet_tris, &tet_pts, None, None, None, None);
        let expected = BBox::new_with(Point::new(),
                                      Point::new_with(1.0, 1.0, 1.0));

        assert_eq!(mesh.object_bound(), expected);
        assert_eq!(Mesh::new(Transform::new(), Transform::new(), false,
                             &tet_tris, &tet_pts, None, None, None, None).object_bound(),
                   expected);
    }

    #[test]
    fn it_has_world_space_bounds() {
        let xf = Transform::rotate_y(90.0);
        let mesh = Mesh::new(xf.clone(), xf.inverse(), false,
                             &tet_tris, &tet_pts, None, None, None, None);

        let expected = BBox::new_with(Point::new_with(0.0, 0.0, -1.0),
                                      Point::new_with(1.0, 1.0, 0.0));


        assert!((mesh.world_bound().p_min - expected.p_min).length_squared() < 1e-6);
        assert!((mesh.world_bound().p_max - expected.p_max).length_squared() < 1e-6);
        assert_eq!(Mesh::new(Transform::new(), Transform::new(), false,
                             &tet_tris, &tet_pts, None, None, None, None).world_bound(),
                   BBox::new_with(Point::new(), Point::new_with(1.0, 1.0, 1.0)));
    }

    #[test]
    fn it_can_be_refined_to_triangles() {
        let xf = Transform::rotate_y(90.0);
        let mesh = Mesh::new(xf.clone(), xf.inverse(), false,
                             &tet_tris, &tet_pts, None, None, None, None);
        let tris = mesh.to_tris();

        assert_eq!(tris.len(), 4);
        assert_eq!(tris[3].v, [2, 3, 0]);
        assert_eq!(tris[2].v, [2, 1, 0]);
        assert_eq!(tris[1].v, [1, 3, 0]);
        assert_eq!(tris[0].v, [3, 2, 1]);
    }

    #[test]
    fn its_triangles_have_object_space_bounds() {
        let xf = Transform::rotate_y(90.0);
        let mesh = Mesh::new(xf.clone(), xf.inverse(), false,
                             &tet_tris, &tet_pts, None, None, None, None);
        let tris = mesh.to_tris();
        assert_eq!(tris[3].object_bound(),
                   BBox::new_with(Point::new(), Point::new_with(0.0, 1.0, 1.0)));
        assert_eq!(tris[2].object_bound(),
                   BBox::new_with(Point::new(), Point::new_with(1.0, 1.0, 0.0)));
        assert_eq!(tris[1].object_bound(),
                   BBox::new_with(Point::new(), Point::new_with(1.0, 0.0, 1.0)));
        assert_eq!(tris[0].object_bound(),
                   BBox::new_with(Point::new(), Point::new_with(1.0, 1.0, 1.0)));
    }

    #[test]
    fn its_triangles_have_world_space_bounds() {
        let xf = Transform::rotate_y(90.0);
        let mesh = Mesh::new(xf.clone(), xf.inverse(), false,
                             &tet_tris, &tet_pts, None, None, None, None);
        let tris = mesh.to_tris();
        assert!((tris[3].world_bound().p_min - Point::new()).length_squared() < 1e-6);
        assert!((tris[3].world_bound().p_max -
                 Point::new_with(1.0, 1.0, 0.0)).length_squared() < 1e-6);
        assert!((tris[2].world_bound().p_min -
                 Point::new_with(0.0, 0.0, -1.0)).length_squared() < 1e-6);
        assert!((tris[2].world_bound().p_max -
                 Point::new_with(0.0, 1.0, 0.0)).length_squared() < 1e-6);
        assert!((tris[1].world_bound().p_min -
                 Point::new_with(0.0, 0.0, -1.0)).length_squared() < 1e-6);
        assert!((tris[1].world_bound().p_max -
                 Point::new_with(1.0, 0.0, 0.0)).length_squared() < 1e-6);
        assert!((tris[0].world_bound().p_min -
                 Point::new_with(0.0, 0.0, -1.0)).length_squared() < 1e-6);
        assert!((tris[0].world_bound().p_max -
                 Point::new_with(1.0, 1.0, 0.0)).length_squared() < 1e-6);
    }

    #[test]
    fn its_triangles_can_be_intersected() {
        let mesh = Mesh::new(Transform::new(), Transform::new(), false,
                             &tet_tris, &tet_pts, None, None, None, None);
        let tris = mesh.to_tris();

        assert!(tris[0].intersect_p(&Ray::new_with(
            Point::new(), Vector::new_with(1.0, 1.0, 1.0), 0.0)));

        assert!(!tris[0].intersect_p(&Ray::new_with(
            Point::new_with(1.5, 1.5, 1.5),
            Vector::new_with(1.0, 1.0, 1.0), 0.0)));

        assert!(tris[0].intersect_p(&Ray::new_with(
            Point::new_with(1.5, 1.5, 1.5),
            Vector::new_with(-1.0, -1.0, -1.0), 0.0)));

        assert!(!tris[0].intersect_p(&Ray::new_with(
            Point::new_with(1.0, 1.0, -1.0),
            Vector::new_with(-1.0, -1.0, 1.0), 0.0)));
    }

    #[test]
    #[ignore]
    fn its_triangles_have_intersection_information() {
        // !FIXME! Implement this when I know how to actually measure it.
        unimplemented!();
    }
}
