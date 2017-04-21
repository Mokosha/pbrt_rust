use std::f32::consts;
use std::fmt::Debug;
use std::ops::Deref;

use diff_geom::DifferentialGeometry;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Vector;
use geometry::vector::Dot;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

use geometry::vector::spherical_theta;
use geometry::vector::spherical_phi;

mod internal {
    use super::*;

    pub trait TextureMapping2DBase {
        // Returns the s and t coordinates for the point on the given surface,
        // and also returns their differentials:
        //   (s, t, ds/dx, dt/dx, ds/dy, dt/dy)
        fn map_dg(&self, dg: &DifferentialGeometry) ->
            (f32, f32, f32, f32, f32, f32);
    }

    impl<U> TextureMapping2DBase for U where U: Deref<Target = TextureMapping2D> {
        fn map_dg(&self, dg: &DifferentialGeometry)
                  -> (f32, f32, f32, f32, f32, f32) {
            self.deref().map(&dg)
        }
    }
}

pub trait TextureMapping2D:
Debug + Send + Sync + internal::TextureMapping2DBase {
    fn map(&self, &DifferentialGeometry) ->
        (f32, f32, f32, f32, f32, f32);
}

impl<U> TextureMapping2D for U where U:
Debug + Send + Sync + internal::TextureMapping2DBase {
    fn map(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32) {
        self.map_dg(&dg)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct UVMapping2D {
    su: f32,
    sv: f32,
    du: f32,
    dv: f32,
}

impl UVMapping2D {
    pub fn new_with(_su: f32, _sv: f32, _du: f32, _dv: f32) -> UVMapping2D {
        UVMapping2D { su: _su, sv: _sv, du: _du, dv: _dv }
    }

    pub fn new() -> UVMapping2D {
        UVMapping2D::new_with(1.0, 1.0, 0.0, 0.0)
    }
}

impl internal::TextureMapping2DBase for UVMapping2D {
    fn map_dg(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32) {
        let s = self.su * dg.u + self.du;
        let t = self.sv * dg.v + self.dv;
        let dsdx = self.su * dg.dudx;
        let dtdx = self.sv * dg.dvdx;
        let dsdy = self.su * dg.dudy;
        let dtdy = self.sv * dg.dvdy;
        (s, t, dsdx, dtdx, dsdy, dtdy)
    }
}

fn get_circular_differentials<F>(dg: &DifferentialGeometry, mapping: F) ->
    (f32, f32, f32, f32, f32, f32) where F: Fn(&Point) -> (f32, f32) {
        let (s, t) = mapping(&dg.p);

        let delta : f32 = 0.1;
        let deal_with_singularity = |res: f32| {
            if res > 0.5 {
                1.0 - res
            } else if res < -0.5 {
                -(res + 1.0)
            } else {
                res
            }
        };

        let px = &dg.p + delta * &dg.dpdx;
        let (sx, tx) = mapping(&px);
        let dsdx = (sx - s) / delta;
        let dtdx = deal_with_singularity((tx - t) / delta);

        let py = &dg.p + delta * &dg.dpdy;
        let (sy, ty) = mapping(&py);
        let dsdy = (sy - s) / delta;
        let dtdy = deal_with_singularity((ty - t) / delta);

        (s, t, dsdx, dtdx, dsdy, dtdy)
    }

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct SphericalMapping2D {
    world_to_texture: Transform
}

impl SphericalMapping2D {
    pub fn new_with(xf: Transform) -> SphericalMapping2D {
        SphericalMapping2D { world_to_texture: xf }
    }

    pub fn new() -> SphericalMapping2D {
        SphericalMapping2D::new_with(Transform::new())
    }

    fn sphere(&self, p: &Point) -> (f32, f32) {
        let vec = {
            let v = Vector::from(self.world_to_texture.xf(p.clone()));
            if v == Vector::new() {
                Vector::new_with(1.0, 0.0, 0.0)
            } else {
                v.normalize()
            }
        };
        let theta = spherical_theta(&vec);
        let phi = spherical_phi(&vec);
        (theta * consts::FRAC_1_PI, phi * consts::FRAC_1_PI * 0.5)
    }
}

impl internal::TextureMapping2DBase for SphericalMapping2D {
    fn map_dg(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32) {
        get_circular_differentials(dg, |p| { self.sphere(p) })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct CylindricalMapping2D {
    world_to_texture: Transform
}

impl CylindricalMapping2D {
    pub fn new_with(xf: Transform) -> CylindricalMapping2D {
        CylindricalMapping2D { world_to_texture: xf }
    }

    pub fn new() -> CylindricalMapping2D {
        CylindricalMapping2D::new_with(Transform::new())
    }

    fn cylinder(&self, p: &Point) -> (f32, f32) {
        let vec = {
            let v = Vector::from(self.world_to_texture.xf(p.clone()));
            if v == Vector::new() {
                Vector::new_with(1.0, 0.0, 0.0)
            } else {
                v.normalize()
            }
        };
        ((consts::PI + vec.y.atan2(vec.x)) / (2.0 * consts::PI), vec.z)
    }
}

impl internal::TextureMapping2DBase for CylindricalMapping2D {
    fn map_dg(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32) {
        get_circular_differentials(dg, |p| { self.cylinder(p) })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct PlanarMapping2D {
    vs: Vector,
    vt: Vector,
    ds: f32,
    dt: f32
}

impl PlanarMapping2D {
    pub fn new() -> PlanarMapping2D {
        PlanarMapping2D {
            vs: Vector::new_with(1.0, 0.0, 0.0),
            vt: Vector::new_with(0.0, 1.0, 0.0),
            ds: 0.0,
            dt: 0.0
        }
    }

    pub fn new_with(v1: Vector, v2: Vector, d1: f32, d2: f32)
                    -> PlanarMapping2D {
        PlanarMapping2D { vs: v1, vt: v2, ds: d1, dt: d2 }
    }
}

impl internal::TextureMapping2DBase for PlanarMapping2D {
    fn map_dg(&self, dg: &DifferentialGeometry) -> (f32, f32, f32, f32, f32, f32) {
        let vec = Vector::from(dg.p.clone());
        let s = self.ds + vec.dot(&self.vs);
        let t = self.dt + vec.dot(&self.vt);
        let dsdx = self.vs.dot(&dg.dpdx);
        let dtdx = self.vt.dot(&dg.dpdx);
        let dsdy = self.vs.dot(&dg.dpdy);
        let dtdy = self.vt.dot(&dg.dpdy);
        (s, t, dsdx, dtdx, dsdy, dtdy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diff_geom::DifferentialGeometry;

    fn test_uv_mapping_deriv(mapping: UVMapping2D) {
        let mut dg = DifferentialGeometry::new();
        dg.u = 0.5;
        dg.v = 0.1;
        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;

        let (s1, t1, dsdx1, dtdx1, dsdy1, dtdy1) = mapping.map(&dg);

        let dx : f32 = 1.0;
        let dy : f32 = -0.4;
        dg.u = dg.u + dx * dg.dudx + dy * dg.dudy;
        dg.v = dg.v + dx * dg.dvdx + dy * dg.dvdy;

        let (s2, t2, dsdx2, dtdx2, dsdy2, dtdy2) = mapping.map(&dg);

        // The derivatives didn't change between invocations, so neither should
        // the mapping derivatives
        assert_eq!(dsdx1, dsdx2);
        assert_eq!(dsdy1, dsdy2);
        assert_eq!(dtdx1, dtdx2);
        assert_eq!(dtdy1, dtdy2);

        let es : f32 = s1 + dx * dsdx1 + dy * dsdy1;
        let et : f32 = t1 + dx * dtdx1 + dy * dtdy1;

        assert!((es - s2).abs() < 0.001);
        assert!((et - t2).abs() < 0.001);
    }

    fn test_positional_differentials<Mapping : TextureMapping2D>(m: Mapping) {
        let mut dg = DifferentialGeometry::new();
        let (s, t, dsdx, dtdx, dsdy, dtdy) = m.map(&dg);
        dg.u = 0.5;
        dg.v = 0.1;
        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;
        assert_eq!(m.map(&dg), (s, t, dsdx, dtdx, dsdy, dtdy));

        dg.p = Point::new_with(0.3, 1.2, -4.0);
        dg.dpdx = Vector::new_with(0.2, 0.0, -0.3);
        dg.dpdy = Vector::new_with(-0.5, 0.1, 1.3);

        let (s1, t1, dsdx1, dtdx1, dsdy1, dtdy1) = m.map(&dg);

        let dx : f32 = 0.1;
        let dy : f32 = -0.1;
        dg.p = &dg.p + dx * &dg.dpdx + dy * &dg.dpdy;

        let (ns, nt, dsdx2, dtdx2, dsdy2, dtdy2) = m.map(&dg);

        // The derivatives might change based on the position of the point,
        // but they should never change significantly if we're only moving
        // based on the differential.
        assert!((dsdx1 - dsdx2).abs() < 0.01);
        assert!((dsdy1 - dsdy2).abs() < 0.01);
        assert!((dtdx1 - dtdx2).abs() < 0.01);
        assert!((dtdy1 - dtdy2).abs() < 0.01);

        let es : f32 = s1 + dx * dsdx1 + dy * dsdy1;
        let et : f32 = t1 + dx * dtdx1 + dy * dtdy1;

        assert!((es - ns).abs() < 0.001);
        assert!((et - nt).abs() < 0.001);
    }

    #[test]
    fn it_can_create_uv_mapping() {
        assert_eq!(UVMapping2D::new(), UVMapping2D::new_with(1., 1., 0., 0.));
        assert_eq!(SphericalMapping2D::new(),
                   SphericalMapping2D::new_with(Transform::new()));
    }

    #[test]
    fn uv_mapping_can_map_coords() {
        let mapping = UVMapping2D::new();
        let mut dg = DifferentialGeometry::new();
        assert_eq!(mapping.map(&dg), (0.0, 0.0, 0.0, 0.0, 0.0, 0.0));

        dg.u = 0.5;
        dg.v = 0.2;
        assert_eq!(mapping.map(&dg), (0.5, 0.2, 0.0, 0.0, 0.0, 0.0));

        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;
        assert_eq!(mapping.map(&dg), (0.5, 0.2, 10.0, -1.0, 12.0, 0.0));

        test_uv_mapping_deriv(mapping);
    }

    #[test]
    fn uv_mapping_can_scale_coords() {
        let mapping = UVMapping2D::new_with(2.0, 0.5, 1.0, -0.3);
        let mut dg = DifferentialGeometry::new();
        assert_eq!(mapping.map(&dg), (1.0, -0.3, 0.0, 0.0, 0.0, 0.0));        

        dg.p = Point::new_with(0.1, 10.0, -13.0);
        assert_eq!(mapping.map(&dg), (1.0, -0.3, 0.0, 0.0, 0.0, 0.0));

        dg.u = 0.5;
        dg.v = 0.2;
        let expected_t = 0.2 * 0.5 - 0.3;  // Stupid floating point precision
        assert_eq!(mapping.map(&dg), (2.0, expected_t, 0.0, 0.0, 0.0, 0.0));

        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;
        assert_eq!(mapping.map(&dg), (2.0, expected_t, 20.0, -0.5, 24.0, 0.0));

        test_uv_mapping_deriv(mapping);
    }

    #[test]
    fn spherical_mapping_can_map_coords() {
        let mapping = SphericalMapping2D::new();
        let mut dg = DifferentialGeometry::new();
        assert_eq!(mapping.map(&dg), (0.5, 0.0, 0.0, 0.0, 0.0, 0.0));

        // Changing u and v should do nothing
        dg.u = 0.5;
        dg.v = 0.2;
        assert_eq!(mapping.map(&dg), (0.5, 0.0, 0.0, 0.0, 0.0, 0.0));

        // Changing the position should do something
        dg.p = Point::new_with(0.1, 0.2, 0.6);
        let (s, t, _, _, _, _) = mapping.map(&dg);
        assert_ne!(s, 0.0);
        assert_ne!(t, 0.0);

        for i in 1..9 {
            let di = (i as f32) / 10.0;
            dg.p = Point::new_with(0.1 * di, 0.2 * di, 0.6 * di);
            let (ns, nt, _, _, _, _) = mapping.map(&dg);
            assert!((s - ns).abs() < 0.00001);
            assert!((t - nt).abs() < 0.00001);
        }
    }

    #[test]
    fn transformed_spherical_mapping_can_map_coords() {
        let translated_mapping = SphericalMapping2D::new_with(
            Transform::translate(&Vector::new_with(1.0, 2.0, 3.0)));

        let identity_mapping = SphericalMapping2D::new();

        let mut dg_one = DifferentialGeometry::new();
        dg_one.p = Point::new_with(1.0, 2.0, 3.0);

        let mut dg_two = DifferentialGeometry::new();

        assert_eq!(translated_mapping.map(&dg_two),
                   identity_mapping.map(&dg_one));
    }

    #[test]
    fn identity_spherical_mapping_can_produce_differentials() {
        let identity_mapping = SphericalMapping2D::new();
        test_positional_differentials(identity_mapping);
    }

    #[test]
    fn transformed_spherical_mapping_can_produce_differentials() {
        let transformed_mapping = SphericalMapping2D::new_with(
            Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
                * Transform::rotate_x(45.0));

        test_positional_differentials(transformed_mapping);
    }

    #[test]
    fn cylindrical_mapping_can_map_coords() {
        let mapping = CylindricalMapping2D::new();
        let mut dg = DifferentialGeometry::new();
        assert_eq!(mapping.map(&dg), (0.5, 0.0, 0.0, 0.0, 0.0, 0.0));

        // Changing u and v should do nothing
        dg.u = 0.5;
        dg.v = 0.2;
        assert_eq!(mapping.map(&dg), (0.5, 0.0, 0.0, 0.0, 0.0, 0.0));

        // Changing the position should do something
        dg.p = Point::new_with(0.1, 0.2, 0.6);
        let (s, t, _, _, _, _) = mapping.map(&dg);
        assert_ne!(s, 0.0);
        assert_ne!(t, 0.0);

        for i in 1..9 {
            let di = (i as f32) / 10.0;
            dg.p = Point::new_with(0.1 * di, 0.2 * di, 0.6 * di);
            let (ns, nt, _, _, _, _) = mapping.map(&dg);
            assert!((s - ns).abs() < 0.00001);
            assert!((t - nt).abs() < 0.00001);
        }
    }

    #[test]
    fn transformed_cylindrical_mapping_can_map_coords() {
        let translated_mapping = CylindricalMapping2D::new_with(
            Transform::translate(&Vector::new_with(1.0, 2.0, 3.0)));

        let identity_mapping = CylindricalMapping2D::new();

        let mut dg_one = DifferentialGeometry::new();
        dg_one.p = Point::new_with(1.0, 2.0, 3.0);

        let mut dg_two = DifferentialGeometry::new();

        assert_eq!(translated_mapping.map(&dg_two),
                   identity_mapping.map(&dg_one));
    }

    #[test]
    fn identity_cylindrical_mapping_can_produce_differentials() {
        let identity_mapping = CylindricalMapping2D::new();
        test_positional_differentials(identity_mapping);
    }

    #[test]
    fn transformed_cylindrical_mapping_can_produce_differentials() {
        let transformed_mapping = CylindricalMapping2D::new_with(
            Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
                * Transform::rotate_x(45.0));

        test_positional_differentials(transformed_mapping);
    }

    #[test]
    fn identity_planar_mapping_can_produce_differentials() {
        let planar_mapping = PlanarMapping2D::new();
        test_positional_differentials(planar_mapping);
    }

    #[test]
    fn transformed_planar_mapping_can_produce_differentials() {
        let planar_mapping = PlanarMapping2D::new_with(
            Vector::new_with(1.0, 2.0, 3.0),
            Vector::new_with(-3.0, 0.0, -1.2),
            3.2, -1000.0);

        test_positional_differentials(planar_mapping);
    }
}
