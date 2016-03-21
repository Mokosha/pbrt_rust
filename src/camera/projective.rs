use camera::CameraBase;
use camera::CameraSample;
use camera::film::Film;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Vector;
use ray::Ray;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;

// !FIXME! This doesn't belong here!
fn concentric_sample_disk(x: f32, y: f32) -> (f32, f32) { (x, y) }

#[derive(Debug, Clone)]
pub struct Projection {
    camera_to_screen: Transform,
    raster_to_screen: Transform,
    screen_to_raster: Transform,
    raster_to_camera: Transform,

    lens_radius: f32,
    focal_distance: f32
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
            lens_radius: lensr,
            focal_distance: focald
        }
    }

    pub fn camera_to_screen(&self) -> &Transform { &self.camera_to_screen }
    pub fn raster_to_screen(&self) -> &Transform { &self.raster_to_screen }
    pub fn screen_to_raster(&self) -> &Transform { &self.screen_to_raster }
    pub fn raster_to_camera(&self) -> &Transform { &self.raster_to_camera }

    pub fn handle_dof(&self, sample: &CameraSample, ray: &mut Ray) {
        if self.lens_radius <= 0.0 {
            return;
        }

        // Sample point on lens
        let (mut u, mut v) = concentric_sample_disk(sample.lens_u, sample.lens_v);
        u *= self.lens_radius;
        v *= self.lens_radius;

        // Compute point on plane of focus
        let ft = self.focal_distance / ray.d.z;
        let p_focus = ray.point_at(ft);

        // Update ray for effect of lens
        ray.o = Point::new_with(u, v, 0.0);
        ray.d = (p_focus - &ray.o).normalize();
    }
}

#[cfg(test)]
mod tests {

    #[ignore]
    #[test]
    fn it_can_setup_camera_to_screen_transform() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_setup_raster_to_screen_transform() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_setup_screen_to_raster_transform() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_setup_raster_to_camera_transform() {
        unimplemented!()
    }
}
