use camera::CameraBase;
use camera::CameraSample;
use camera::film::Film;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Vector;
use ray::Ray;
use transform::animated::AnimatedTransform;
use transform::transform::Transform;

macro_rules! check_mat {
    ($m1: expr, $m2: expr) => {{
        let x = ($m1).clone();
        let y = ($m2).clone();
        for i in 0..4 {
            for j in 0..4 {
                let diff = (x[i][j] - y[i][j]).abs();
                if diff >= 5e-5 {
                    println!("m1: {:?}", x);
                    println!("m2: {:?}", y);
                    println!("Matrices differ at {:?} by {:?}", (i, j), diff);
                    panic!();
                }
            }
        }
    }}
}

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
    // The screen_window are the camera-space coordinates that define the
    // size of the window. E.g. if the screen_window is [-1, 1, -1, 1], then
    // the extent in each direction is two units, meaning that the size of
    // the window in camera space is [-2, 0, 0, -2]
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
            Transform::translate(&Vector::new_with(
                -screen_window[0], -screen_window[3], 0.0));
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
    use super::*;
    use camera::film::Film;
    use filter::Filter;
    use transform::transform::ApplyTransform;
    use transform::transform::Transform;
    use geometry::vector::Vector;

    // Test a simple orthographic projection with a near and far plane at one
    // and two respectively....
    fn mk_ortho() -> Transform {
        Transform::translate(&Vector::new_with(0.0, 0.0, -1.0))
    }

    fn mk_film() -> Film {
        Film::image(640, 480, Filter::mean(3.0, 3.0),
                    [0.0, 1.0, 0.0, 1.0], "".to_string(), false)
    }

    #[test]
    fn it_can_setup_camera_to_screen_transform() {
        let film = mk_film();
        let mut mat = mk_ortho();
        let mut p = Projection::new(&film, mat.clone(), [0.0, 640.0, 0.0, 480.0],
                                    1.0, 1.0);

        assert_eq!(mat, p.camera_to_screen().clone());

        // Test other transforms
        mat = Transform::new();
        p = Projection::new(&film, mat.clone(), [0.0, 640.0, 0.0, 480.0],
                            1.0, 1.0);
        assert_eq!(mat, p.camera_to_screen.clone());

        mat = Transform::from([[1.0, 0.0, 0.0, 0.0],
                               [0.0, 1.0, 0.0, 0.0],
                               [0.0, 0.0, 2.0, -2.0],
                               [0.0, 0.0, 1.0, 0.0]]);
        p = Projection::new(&film, mat.clone(), [0.0, 640.0, 0.0, 480.0],
                            1.0, 1.0);
        assert_eq!(mat, p.camera_to_screen.clone());
    }

    #[test]
    fn it_can_setup_raster_to_screen_transform() {
        let film = mk_film();
        let mut m = mk_ortho();
        let mut p = Projection::new(&film, m.clone(), [0.0, 640.0, 0.0, 480.0],
                                    1.0, 1.0);

        assert_eq!(p.raster_to_screen().clone(),

                   // y-axis is flipped, so we need to account for that
                   Transform::from([[1.0, 0.0, 0.0, 0.0],
                                    [0.0, -1.0, 0.0, 480.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]]));

        // Changing the projective transform shouldn't change the raster
        // to screen transform
        m = Transform::new();
        p = Projection::new(&film, m.clone(), [0.0, 640.0, 0.0, 480.0],
                            1.0, 1.0);

        assert_eq!(p.raster_to_screen().clone(),

                   // y-axis is flipped, so we need to account for that
                   Transform::from([[1.0, 0.0, 0.0, 0.0],
                                    [0.0, -1.0, 0.0, 480.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]]));

        // If we use a smaller window though, like from 0-1, it should have
        // some scaling
        p = Projection::new(&film, m.clone(), [0.0, 1.0, 0.0, 1.0], 1.0, 1.0);

        check_mat!(p.raster_to_screen().clone(),

                   // y-axis is flipped, so we need to account for that
                   Transform::from([[1.0 / 640.0, 0.0, 0.0, 0.0],
                                    [0.0, -1.0 / 480.0, 0.0, 1.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]]));
    }

    // Same as previous test except matrices are inverted
    #[test]
    fn it_can_setup_screen_to_raster_transform() {
        let film = mk_film();
        let mut m = mk_ortho();
        let mut p = Projection::new(&film, m.clone(), [0.0, 640.0, 0.0, 480.0],
                                    1.0, 1.0);

        assert_eq!(p.screen_to_raster().clone(),

                   // y-axis is flipped, so we need to account for that
                   Transform::from([[1.0, 0.0, 0.0, 0.0],
                                    [0.0, -1.0, 0.0, 480.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]])
                   .invert());

        // Changing the projective transform shouldn't change the raster
        // to screen transform
        m = Transform::new();
        p = Projection::new(&film, m.clone(), [0.0, 640.0, 0.0, 480.0],
                            1.0, 1.0);

        assert_eq!(p.screen_to_raster().clone(),

                   // y-axis is flipped, so we need to account for that
                   Transform::from([[1.0, 0.0, 0.0, 0.0],
                                    [0.0, -1.0, 0.0, 480.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]])
                   .invert());

        // If we use a smaller window though, like from 0-1, it should have
        // some scaling
        p = Projection::new(&film, m.clone(), [0.0, 1.0, 0.0, 1.0], 1.0, 1.0);

        check_mat!(p.screen_to_raster().clone(),

                   // y-axis is flipped, so we need to account for that
                   Transform::from([[1.0 / 640.0, 0.0, 0.0, 0.0],
                                    [0.0, -1.0 / 480.0, 0.0, 1.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0]])
                   .invert());
    }

    #[test]
    fn it_can_setup_raster_to_camera_transform() {
        let film = mk_film();
        let mut m = mk_ortho();
        let mut r2c = Projection::new(&film, m.clone(), [0.0, 1.0, 0.0, 1.0],
                                      1.0, 1.0).raster_to_camera().clone();

        assert_eq!(r2c.xf(Vector::new_with(160.0, 120.0, 1.0)),
                   Vector::new_with(0.25, -0.25, 1.0));
        assert!((r2c.xf(Vector::new_with(480.0, 360.0, 0.0)) -
                 Vector::new_with(0.75, -0.75, 0.0)).length_squared() < 1e-5);

        r2c = Projection::new(&film, m.clone(), [-1.0, 1.0, -1.0, 1.0],
                              1.0, 1.0).raster_to_camera().clone();

        assert_eq!(r2c.xf(Vector::new_with(160.0, 120.0, 1.0)),
                   Vector::new_with(0.5, -0.5, 1.0));
        assert!((r2c.xf(Vector::new_with(480.0, 360.0, 0.0)) -
                 Vector::new_with(1.5, -1.5, 0.0)).length_squared() < 1e-5);
    }

    #[ignore]
    #[test]
    fn it_can_adjust_rays_for_dof() {
    }
}
