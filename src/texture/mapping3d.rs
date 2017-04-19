use diff_geom::DifferentialGeometry;
use geometry::point::Point;
use geometry::vector::Vector;
use transform::transform::ApplyTransform;
use transform::transform::Transform;

pub trait TextureMapping3D {
    // Returns the 3d coordinates for the point providing the r, s, t,
    // coordinates of the texture mapping. Also returns their partial
    // derivatives in the x and y direction.
    fn map(&self, dg: &DifferentialGeometry) -> (Point, Vector, Vector);
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct IdentityMapping3D {
    world_to_texture: Transform
}

impl IdentityMapping3D {
    pub fn new_with(xf: Transform) -> IdentityMapping3D {
        IdentityMapping3D { world_to_texture: xf }
    }

    pub fn new() -> IdentityMapping3D {
        IdentityMapping3D::new_with(Transform::new())
    }
}

impl TextureMapping3D for IdentityMapping3D {
    fn map(&self, dg: &DifferentialGeometry) -> (Point, Vector, Vector) {
        let dpdx = self.world_to_texture.t(&dg.dpdx);
        let dpdy = self.world_to_texture.t(&dg.dpdy);
        (self.world_to_texture.t(&dg.p), dpdx, dpdy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diff_geom::DifferentialGeometry;

    fn test_positional_differentials<Mapping : TextureMapping3D>(m: Mapping) {
        let mut dg = DifferentialGeometry::new();
        let (p, dpdx, dpdy) = m.map(&dg);
        dg.u = 0.5;
        dg.v = 0.1;
        dg.dudx = 10.0;
        dg.dudy = 12.0;
        dg.dvdx = -1.0;
        dg.dvdy = 0.0;
        assert_eq!(m.map(&dg), (p.clone(), dpdx, dpdy));

        dg.p = Point::new_with(0.3, 1.2, -4.0);
        dg.dpdx = Vector::new_with(0.2, 0.0, -0.3);
        dg.dpdy = Vector::new_with(-0.5, 0.1, 1.3);

        let (p1, dpdx1, dpdy1) = m.map(&dg);

        let dx : f32 = 0.1;
        let dy : f32 = -0.1;
        dg.p = &dg.p + dx * &dg.dpdx + dy * &dg.dpdy;

        let (np, dpdx2, dpdy2) = m.map(&dg);

        // The derivatives might change based on the position of the point,
        // but they should never change significantly if we're only moving
        // based on the differential.
        assert_eq!(dpdx1, dpdx2);
        assert_eq!(dpdy1, dpdy2);

        let ep = p1 + dx * dpdx1 + dy * dpdy1;
        assert!(Vector::from(ep - np).length_squared() < 0.001);
    }

    #[test]
    fn identity_mapping_can_produce_differentials() {
        let identity_mapping = IdentityMapping3D::new();
        test_positional_differentials(identity_mapping);
    }

    #[test]
    fn transformed_identity_mapping_can_produce_differentials() {
        let identity_mapping = IdentityMapping3D::new_with(
            Transform::translate(&Vector::new_with(1.0, 2.0, 3.0))
                * Transform::rotate_x(45.0));

        test_positional_differentials(identity_mapping);
    }
}
