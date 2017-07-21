use std::sync::Arc;

use diff_geom::DifferentialGeometry;
use texture::Texture;
use texture::internal::TextureBase;
use texture::mapping2d::TextureMapping2D;
use utils::Lerp;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
enum CheckerboardAA {
    NONE,
    CLOSEDFORM
}

#[derive(Debug)]
pub struct CheckerboardTexture<T> {
    mapping: Box<TextureMapping2D>,
    tex1: Arc<Texture<T>>,
    tex2: Arc<Texture<T>>,
    aa_method: CheckerboardAA
}

impl<T> CheckerboardTexture<T> {
    pub fn new(mapping: Box<TextureMapping2D>, t1: Arc<Texture<T>>,
               t2: Arc<Texture<T>>) -> CheckerboardTexture<T> {
        CheckerboardTexture { mapping: mapping, tex1: t1, tex2: t2,
                              aa_method: CheckerboardAA::NONE }
    }

    pub fn new_antialiased(mapping: Box<TextureMapping2D>, t1: Arc<Texture<T>>,
                           t2: Arc<Texture<T>>) -> CheckerboardTexture<T> {
        CheckerboardTexture { mapping: mapping, tex1: t1, tex2: t2,
                              aa_method: CheckerboardAA::CLOSEDFORM }
    }

    fn point_sample(&self, dg: &DifferentialGeometry, s: f32, t: f32) -> T {
        if ((s.floor() as i32) + (t.floor() as i32)) % 2 == 0 {
            self.tex1.evaluate(dg)
        } else {
            self.tex2.evaluate(dg)
        }
    }
}

impl<T> TextureBase<T> for CheckerboardTexture<T> where T: Lerp<f32> {
    fn eval(&self, dg: &DifferentialGeometry) -> T {
        let (s, t, dsdx, dtdx, dsdy, dtdy) = self.mapping.map(dg);
        match self.aa_method {
            CheckerboardAA::NONE => self.point_sample(dg, s, t),
            CheckerboardAA::CLOSEDFORM => {
                // Compute closed-form box-filtered checkerboard value
                let ds = dsdx.abs().max(dsdy.abs());
                let dt = dtdx.abs().max(dtdy.abs());
                let (s0, t0) = (s - ds, t - dt);
                let (s1, t1) = (s + ds, t + dt);
                if s0.floor() == s1.floor() && t0.floor() == t1.floor() {
                    self.point_sample(dg, s, t)
                } else {
                    // Apply box filter to checkerboard region
                    let bump_int = |x: f32| {
                        let half_x = x / 2.0;
                        half_x.floor() +
                            2.0 * (half_x - half_x.floor() - 0.5).max(0.0)
                    };

                    let sint = if ds > 0.0 {
                        (bump_int(s1) - bump_int(s0)) / (2.0 * ds)
                    } else {
                        0.0
                    };

                    let tint = if dt > 0.0 {
                        (bump_int(t1) - bump_int(t0)) / (2.0 * dt)
                    } else {
                        0.0
                    };

                    let area_sq =
                        if ds > 1.0 || dt > 1.0 {
                            0.5
                        } else {
                            sint + tint - 2.0 * sint * tint
                        };

                    let t1 = self.tex1.evaluate(dg);
                    let t2 = self.tex2.evaluate(dg);
                    t1.lerp_with(t2, area_sq)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texture::ConstantTexture;
    use texture::mapping2d::PlanarMapping2D;
    use geometry::point::Point;
    use geometry::vector::Vector;

    #[test]
    fn checkerboard_texture_works() {
        let map = Box::new(PlanarMapping2D::new());
        let tex1 = Arc::new(ConstantTexture::new(1.0f32)) as Arc<Texture<f32>>;
        let tex2 = Arc::new(ConstantTexture::new(0.0f32)) as Arc<Texture<f32>>;
        let checker = CheckerboardTexture::new(map, tex1, tex2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.5, 0.5, 0.0);
        assert_eq!(checker.evaluate(&dg), 1.0);

        dg.p = Point::new_with(1.5, 0.5, 0.0);
        assert_eq!(checker.evaluate(&dg), 0.0);

        dg.p = Point::new_with(1.5, 1.5, 0.0);
        assert_eq!(checker.evaluate(&dg), 1.0);
    }

    #[test]
    fn checkerboard_texture_interpolates() {
        let map = Box::new(PlanarMapping2D::new());
        let tex1 = Arc::new(ConstantTexture::new(1.0f32)) as Arc<Texture<f32>>;
        let tex2 = Arc::new(ConstantTexture::new(0.0f32)) as Arc<Texture<f32>>;
        let checker = CheckerboardTexture::new_antialiased(map, tex1, tex2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(1.1, 0.5, 0.0);
        dg.dpdx = Vector::new_with(0.2, 0.0, 0.0);
        assert!((checker.evaluate(&dg) - 0.25).abs() < 0.001); 

        dg.dpdx = Vector::new_with(0.2, 0.2, 0.0);
        assert!((checker.evaluate(&dg) - 0.25).abs() < 0.001);

        dg.p = Point::new_with(1.1, 0.9, 0.0);
        dg.dpdx = Vector::new_with(0.2, 0.0, 0.0);
        dg.dpdy = Vector::new_with(0.0, 0.2, 0.0);
        let expected: f32 = 2.0 * (0.25 * 0.75);
        assert!((checker.evaluate(&dg) - expected).abs() < 0.001);
    }
}
