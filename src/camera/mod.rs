pub mod film;

use camera::film::Film;
use ray::Ray;
use ray::RayDifferential;
use sampler::Sample;
use spectrum::Spectrum;
use transform::animated::AnimatedTransform;

#[derive(Debug, Clone)]
pub struct CameraSample {
    image_x: usize,
    image_y: usize
}

impl CameraSample {
    pub fn from_sample(s: &Sample) -> CameraSample {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    film: Film,
    cam_to_world: AnimatedTransform,
    shutter_open: f32,
    shutter_close: f32
}

impl Camera {
    pub fn new(film: Film, cam2world: AnimatedTransform,
               sopen: f32, sclose: f32) -> Camera {
        Camera {
            film: film,
            cam_to_world: cam2world,
            shutter_open: sopen,
            shutter_close: sclose
        }
    }

    pub fn film(&self) -> &Film { &(self.film) }
    pub fn film_mut(&mut self) -> &mut Film { &mut (self.film) }

    pub fn generate_ray(&self, sample: &CameraSample) -> (f32, Ray) {
        unimplemented!();
    }

    pub fn generate_ray_differential(&self, sample: &CameraSample)
                                     -> (f32, RayDifferential) {
        let (wt, r) = self.generate_ray(sample);
        let mut rd = RayDifferential::new();

        // Find ray after shifting one pixel in the x direction
        let (wtx, rx) = {
            let mut s = sample.clone();
            s.image_x += 1;
            self.generate_ray(&s)
        };

        // Find ray after shifting one pixel in the y direction
        let (wty, ry) = {
            let mut s = sample.clone();
            s.image_y += 1;
            self.generate_ray(&s)
        };

        rd.ray = r;
        rd.rx_origin = rx.o.clone();
        rd.rx_dir = rx.d.clone();
        rd.ry_origin = ry.o.clone();
        rd.ry_dir = ry.d.clone();

        if wtx == 0.0 || wty == 0.0 {
            return (0.0f32, rd);
        }

        rd.has_differentials = true;
        (wt, rd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
