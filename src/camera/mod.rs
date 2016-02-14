mod projective;
pub mod film;

use camera::film::Film;
use camera::projective::Projection;
use ray::Ray;
use ray::RayDifferential;
use sampler::Sample;
use spectrum::Spectrum;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;

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
pub struct CameraBase {
    film: Film,
    cam_to_world: AnimatedTransform,
    shutter_open: f32,
    shutter_close: f32
}

impl CameraBase {
    fn new(film: Film, cam2world: AnimatedTransform,
           sopen: f32, sclose: f32) -> CameraBase {
        CameraBase {
            film: film,
            cam_to_world: cam2world,
            shutter_open: sopen,
            shutter_close: sclose
        }
    }
}

#[derive(Debug, Clone)]
pub enum Camera {
    Perspective {
        base: CameraBase,
        proj: Projection
    }
}

impl Camera {
    pub fn perspective(cam2world: AnimatedTransform, proj: Transform,
                       screen_window: [f32; 4], sopen: f32, sclose: f32,
                       lensr: f32, focald: f32, film: Film) -> Camera {
        let p = Projection::new(&film, proj, screen_window, lensr, focald);
        let b = CameraBase::new(film, cam2world, sopen, sclose);
        Camera::Perspective { base: b, proj: p }
    }

    pub fn base(&self) -> &CameraBase {
        match self {
            &Camera::Perspective { ref base, .. } => { base }
        }
    }

    fn base_mut(&mut self) -> &mut CameraBase {
        match self {
            &mut Camera::Perspective { ref mut base, .. } => { base }
        }
    }

    pub fn film(&self) -> &Film { &(self.base().film) }
    pub fn film_mut(&mut self) -> &mut Film { &mut self.base_mut().film }

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
