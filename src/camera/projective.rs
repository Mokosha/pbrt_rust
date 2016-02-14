use camera::CameraBase;
use camera::film::Film;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;

#[derive(Debug, Clone)]
pub struct ProjectiveCamera {
    base: CameraBase,
    camera_to_screen: Transform,
    raster_to_camera: Transform
}

impl ProjectiveCamera {
    pub fn new(cam2world: AnimatedTransform, proj: Transform,
               screen_window: [f32; 4], sopen: f32, sclose: f32,
               lensr: f32, focald: f32, film: Film) -> ProjectiveCamera {
        // Initialize depth of field parameters
        // Compute projective camera transformations
        let cam_to_screen = proj.clone();
        let raster_to_cam = {
            // Compute projective camera screen transformations
            let raster_to_screen = Transform::new();
            cam_to_screen.inverse() * raster_to_screen
        };

        ProjectiveCamera {
            base: CameraBase::new(film, cam2world, sopen, sclose),
            camera_to_screen: cam_to_screen,
            raster_to_camera: raster_to_cam
        }
    }

    pub fn base(&self) -> &CameraBase { &(self.base) }
    pub fn base_mut(&mut self) -> &mut CameraBase { &mut (self.base) }
}
