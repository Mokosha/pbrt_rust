mod projective;
pub mod film;

use camera::film::Film;
use camera::projective::Projection;
use geometry::point::Point;
use geometry::normal::Normalize;
use geometry::vector::Vector;
use ray::Ray;
use ray::RayDifferential;
use sampler::Sample;
use spectrum::Spectrum;
use transform::animated::AnimatedTransform;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Lerp;
use utils::Degrees;

#[derive(Debug, Clone)]
pub struct CameraSample {
    image_x: usize,
    image_y: usize,
    lens_u: f32,
    lens_v: f32,
    time: f32
}

impl CameraSample {
    pub fn new(x: usize, y: usize, lu: f32, lv: f32, t: f32) -> CameraSample {
        CameraSample {
            image_x: x,
            image_y: y,
            lens_u: lu,
            lens_v: lv,
            time: t
        }
    }

    pub fn empty() -> CameraSample {
        CameraSample::new(0, 0, 0.0, 0.0, 0.0)
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
        proj: Projection,
        dx_camera: Vector,
        dy_camera: Vector
    },

    Environment {
        base: CameraBase
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
            dx_camera: p.raster_to_camera().xf(Vector::right()),
            dy_camera: p.raster_to_camera().xf(Vector::up()),
        }
    }

    pub fn perspective(cam2world: AnimatedTransform, screen_window: [f32; 4],
                       sopen: f32, sclose: f32, lensr: f32, focald: f32,
                       fov: f32, film: Film) -> Camera {
        let znear: f32 = 1e-2f32;
        let zfar: f32 = 1000.0f32;
        let persp = {
            // Perform projective divide
            let p = Transform::from([[1.0, 0.0, 0.0, 0.0],
                                     [0.0, 1.0, 0.0, 0.0],
                                     [0.0, 0.0, zfar / (zfar - znear),
                                      -(zfar * znear) / (zfar - znear)],
                                     [0.0, 0.0, 1.0, 0.0]]);

            // Scale to canonical viewing volume
            let inv_tan_ang = 1.0 / (fov.as_radians() / 2.0).tan();
            Transform::scale(inv_tan_ang, inv_tan_ang, 1.0) * p
        };

        let p = Projection::new(&film, persp, screen_window, lensr, focald);
        let b = CameraBase::new(film, cam2world, sopen, sclose);

        Camera::Perspective {
            base: b,
            proj: p.clone(),
            // Compute differential changes in origin for perspective camera rays
            dx_camera: p.raster_to_camera().xf(Vector::right()) -
                p.raster_to_camera().xf(Vector::new()),
            dy_camera: p.raster_to_camera().xf(Vector::up()) -
                p.raster_to_camera().xf(Vector::new())
        }
    }

    pub fn environment(cam2world: AnimatedTransform, sopen: f32, sclose: f32,
                       film: Film) -> Camera {
        Camera::Environment {
            base: CameraBase::new(film, cam2world, sopen, sclose)
        }
    }

    pub fn base(&self) -> &CameraBase {
        match self {
            &Camera::Perspective { ref base, .. } => { base },
            &Camera::Orthographic { ref base, .. } => { base },
            &Camera::Environment { ref base, .. } => { base }
        }
    }

    fn base_mut(&mut self) -> &mut CameraBase {
        match self {
            &mut Camera::Perspective { ref mut base, .. } => { base },
            &mut Camera::Orthographic { ref mut base, .. } => { base },
            &mut Camera::Environment { ref mut base, .. } => { base }
        }
    }

    fn proj(&self) -> Option<&Projection> {
        match self {
            &Camera::Perspective { ref proj, .. } => { Some(proj) },
            &Camera::Orthographic { ref proj, .. } => { Some(proj) },
            &Camera::Environment { .. } => None
        }
    }

    fn generate_base_ray(&self, sample: &CameraSample) -> Ray {
        // Generate raster and camera samples
        let p_camera = self.proj().map(|proj| {
            let p_raster = Point::new_with(
                sample.image_x as f32, sample.image_y as f32, 0.0);
            proj.raster_to_camera().xf(p_raster)
        });

        match self {
            &Camera::Orthographic { .. } =>
                Ray::new_with(p_camera.unwrap(), Vector::forward(), 0.0),
            &Camera::Perspective { .. } =>
                Ray::new_with(Point::new(),
                              Vector::from(p_camera.unwrap()).normalize(), 0.0),
            &Camera::Environment { .. } => {
                let theta = ::std::f32::consts::PI *
                    ((sample.image_y as f32) / (self.film().y_res() as f32));
                let phi = 2.0 * ::std::f32::consts::PI *
                    ((sample.image_x as f32) / (self.film().x_res() as f32));

                Ray::new_with(Point::new(),
                              Vector::new_with(theta.sin() * phi.cos(),
                                               theta.cos(),
                                               theta.sin() * phi.sin()),
                              0.0)
            }
        }
    }

    pub fn film(&self) -> &Film { &(self.base().film) }
    pub fn film_mut(&mut self) -> &mut Film { &mut self.base_mut().film }

    pub fn generate_ray(&self, sample: &CameraSample) -> (f32, Ray) {
        let mut ray = self.generate_base_ray(sample);

        // Modify ray for depth of field
        if let Some(proj) = self.proj() {
            proj.handle_dof(sample, &mut ray);
        }

        ray.set_time(self.base().shutter_open.lerp(&self.base().shutter_close, sample.time));
        (1.0, self.base().cam_to_world.xf(ray))
    }

    pub fn generate_ray_differential(&self, sample: &CameraSample)
                                     -> (f32, RayDifferential) {
        let mut rd = RayDifferential::new();
        rd.has_differentials = true;
        rd.ray = self.generate_base_ray(sample);

        match self {
            &Camera::Orthographic { ref base, ref dx_camera, ref dy_camera, .. } => {
                rd.rx_origin = &rd.ray.o + dx_camera;
                rd.ry_origin = &rd.ray.o + dy_camera;
                rd.rx_dir = rd.ray.d.clone();
                rd.ry_dir = rd.ray.d.clone();
            },

            &Camera::Perspective { ref base, ref dx_camera, ref dy_camera, .. } => {
                // Generate raster and camera samples
                let p_raster = Point::new_with(
                    sample.image_x as f32, sample.image_y as f32, 0.0);
                let p_camera = self.proj().unwrap().raster_to_camera().xf(p_raster);

                rd.rx_origin = rd.ray.o.clone();
                rd.ry_origin = rd.ray.o.clone();
                rd.rx_dir = (Vector::from(p_camera.clone()) + dx_camera).normalize();
                rd.ry_dir = (Vector::from(p_camera.clone()) + dy_camera).normalize();
            },

            &Camera::Environment { .. } => {
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

                rd.rx_origin = rx.o.clone();
                rd.rx_dir = rx.d.clone();
                rd.ry_origin = ry.o.clone();
                rd.ry_dir = ry.d.clone();

                if wtx == 0.0 || wty == 0.0 {
                    return (0.0f32, rd);
                }
            }
        }

        // Modify ray for depth of field
        if let Some(proj) = self.proj() {
            proj.handle_dof(sample, &mut rd.ray);
        }

        rd.ray.set_time(self.base().shutter_open.lerp(&self.base().shutter_close, sample.time));
        (1.0, self.base().cam_to_world.xf(rd))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
