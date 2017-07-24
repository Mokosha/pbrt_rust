use std::sync::Arc;

use diff_geom::DifferentialGeometry;
use texture::internal::TextureBase;
use texture::mapping2d::TextureMapping2D;
use texture::noise::noise;
use texture::Texture;

#[derive(Debug)]
pub struct DotsTexture<T> {
    mapping: Box<TextureMapping2D>,
    inside_dot: Arc<Texture<T>>,
    outside_dot: Arc<Texture<T>>
}

impl<T> DotsTexture<T> {
    pub fn new(mapping: Box<TextureMapping2D>, t1: Arc<Texture<T>>,
               t2: Arc<Texture<T>>) -> DotsTexture<T> {
        DotsTexture { mapping: mapping, inside_dot: t1, outside_dot: t2 }
    }
}

impl<T> TextureBase<T> for DotsTexture<T> {
    fn eval(&self, dg: &DifferentialGeometry) -> T {
        let (s, t, _, _, _, _) = self.mapping.map(dg);
        let s_cell = (s + 0.5).floor();
        let t_cell = (t + 0.5). floor();

        if noise(s_cell + 0.5, t_cell + 0.5, 0.5) > 0.0 {
            let radius = 0.35;
            let max_shift = 0.5 - radius;
            let s_center = s_cell + max_shift * noise(
                s_cell + 1.5, t_cell + 2.8, 0.5);
            let t_center = t_cell + max_shift * noise(
                s_cell + 4.5, t_cell + 9.8, 0.5);
            let ds = s - s_center;
            let dt = t - t_center;
            if ds * ds + dt * dt < radius * radius {
                self.inside_dot.evaluate(dg)
            } else {
                self.outside_dot.evaluate(dg)
            }
        } else {
            self.outside_dot.evaluate(dg)
        }
    }
}
