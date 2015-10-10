use ray::RayDifferential;
use sampler::Sample;

#[derive(Debug, Clone)]
pub struct Film {
    x_resolution : i32,
    y_resolution : i32
}

impl Film {
    pub fn x_res(&self) -> i32 { self.x_resolution }
    pub fn y_res(&self) -> i32 { self.y_resolution }
    pub fn num_pixels(&self) -> i32 { self.x_res() * self.y_res() }
}

#[derive(Debug, Clone)]
pub struct Camera {
    film : Film
}

impl Camera {
    pub fn new(x_res: i32, y_res: i32) -> Camera{
        Camera {
            film: Film {
                x_resolution: x_res,
                y_resolution: y_res
            }
        }
    }

    pub fn film(&self) -> &Film { &(self.film) }

    pub fn generate_ray_differential(&self, sample: &Sample)
                                     -> (f32, RayDifferential) {
        (0.0f32, RayDifferential)
    }
}
