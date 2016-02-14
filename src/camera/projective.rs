use camera::CameraBase;
use camera::film::Film;
use geometry::vector::Vector;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;

#[derive(Debug, Clone)]
pub struct Projection {
    camera_to_screen: Transform,
    raster_to_screen: Transform,
    screen_to_raster: Transform,
    raster_to_camera: Transform
}

impl Projection {
    pub fn new(film: &Film, proj: Transform, screen_window: [f32; 4],
               lensr: f32, focald: f32) -> Projection {
        // Initialize depth of field parameters
        // Compute projective camera transformations
        let cam_to_screen = proj.clone();

        // Compute projective camera screen transformations
        let screen_to_raster =
            Transform::scale(film.x_res() as f32, film.y_res() as f32, 1.0) *
            Transform::scale(1.0 / (screen_window[1] - screen_window[0]),
                             1.0 / (screen_window[2] - screen_window[3]), 1.0) *
            Transform::translate(&Vector::new_with(-screen_window[0], -screen_window[3], 0.0));
        let raster_to_screen = screen_to_raster.inverse();
        let raster_to_cam = cam_to_screen.inverse() * raster_to_screen.clone();

        Projection {
            camera_to_screen: cam_to_screen,
            raster_to_screen: raster_to_screen,
            screen_to_raster: screen_to_raster,
            raster_to_camera: raster_to_cam,
        }
    }

    pub fn camera_to_screen(&self) -> &Transform { &self.camera_to_screen }
    pub fn raster_to_screen(&self) -> &Transform { &self.raster_to_screen }
    pub fn screen_to_raster(&self) -> &Transform { &self.screen_to_raster }
    pub fn raster_to_camera(&self) -> &Transform { &self.raster_to_camera }
}
