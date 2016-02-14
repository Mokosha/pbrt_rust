mod projective;
pub mod film;

use camera::film::Film;
use camera::projective::Projection;
use geometry::point::Point;
use geometry::vector::Vector;
use ray::Ray;
use ray::RayDifferential;
use sampler::Sample;
use spectrum::Spectrum;
use transform::animated::AnimatedTransform;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Lerp;

#[derive(Debug, Clone)]
pub struct CameraSample {
    image_x: usize,
    image_y: usize,
    time: f32
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
    Orthographic {
        base: CameraBase,
        proj: Projection,
        dx_camera: Vector,
        dy_camera: Vector
    },

    Perspective {
        base: CameraBase,
        proj: Projection
    }
}

impl Camera {
    pub fn orthographic(cam2world: AnimatedTransform, screen_window: [f32; 4],
                        sopen: f32, sclose: f32, lensr: f32, focald: f32,
                        film: Film) -> Camera {
        let zfar: f32 = 1.0;
        let znear: f32 = 0.0;
        let ortho = Transform::scale(1.0, 1.0, 1.0 / (zfar - znear)) *
            Transform::translate(&Vector::new_with(0.0, 0.0, -znear));

        let p = Projection::new(&film, ortho, screen_window, lensr, focald);
        let b = CameraBase::new(film, cam2world, sopen, sclose);

        Camera::Orthographic {
            base: b,
            proj: p.clone(),
            // Compute differential changes in origin for ortho camera rays
            dx_camera: p.raster_to_camera().xf(Vector::new_with(1.0, 0.0, 0.0)),
            dy_camera: p.raster_to_camera().xf(Vector::new_with(0.0, 1.0, 0.0)),
        }
    }

    pub fn perspective(cam2world: AnimatedTransform, proj: Transform,
                       screen_window: [f32; 4], sopen: f32, sclose: f32,
                       lensr: f32, focald: f32, film: Film) -> Camera {
        unimplemented!();
        let p = Projection::new(&film, proj, screen_window, lensr, focald);
        let b = CameraBase::new(film, cam2world, sopen, sclose);
        Camera::Perspective { base: b, proj: p }
    }

    pub fn base(&self) -> &CameraBase {
        match self {
            &Camera::Perspective { ref base, .. } => { base },
            &Camera::Orthographic { ref base, .. } => { base }
        }
    }

    fn base_mut(&mut self) -> &mut CameraBase {
        match self {
            &mut Camera::Perspective { ref mut base, .. } => { base },
            &mut Camera::Orthographic { ref mut base, .. } => { base }
        }
    }

    pub fn film(&self) -> &Film { &(self.base().film) }
    pub fn film_mut(&mut self) -> &mut Film { &mut self.base_mut().film }

    pub fn generate_ray(&self, sample: &CameraSample) -> (f32, Ray) {
        match self {
            &Camera::Orthographic { ref base, ref proj, .. } => {
                // Generate raster and camera samples
                let p_raster = Point::new_with(sample.image_x as f32,
                                               sample.image_y as f32,
                                               0.0);
                let p_camera = proj.raster_to_camera().xf(p_raster);

                let mut ray = Ray::new_with(p_camera, Vector::forward(), 0.0);
                ray.set_time(base.shutter_open.lerp(&base.shutter_close, sample.time));
                (1.0, base.cam_to_world.xf(ray))
            },
            _ => unimplemented!()
        }
    }

    pub fn generate_ray_differential(&self, sample: &CameraSample)
                                     -> (f32, RayDifferential) {
        match self {
            &Camera::Orthographic { ref base, ref dx_camera, ref dy_camera, .. } => {
                let (wt, r) = self.generate_ray(sample);
                let mut rd = RayDifferential::new();
                rd.ray = r;
                rd.rx_origin = &rd.ray.o + dx_camera;
                rd.ry_origin = &rd.ray.o + dy_camera;
                rd.rx_dir = rd.ray.d.clone();
                rd.ry_dir = rd.ray.d.clone();
                rd.has_differentials = true;
                (wt, base.cam_to_world.xf(rd))
            },
            _ => {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
