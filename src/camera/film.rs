use camera::CameraSample;
use spectrum::Spectrum;

#[derive(Debug, Clone)]
pub struct Film {
    x_resolution : i32,
    y_resolution : i32
}

impl Film {
    pub fn new(xres: i32, yres: i32) -> Film {
        Film { x_resolution: xres, y_resolution: yres }
    }

    pub fn x_res(&self) -> i32 { self.x_resolution }
    pub fn y_res(&self) -> i32 { self.y_resolution }
    pub fn num_pixels(&self) -> i32 { self.x_res() * self.y_res() }
    pub fn add_sample(&mut self, sample: &CameraSample, ls: &Spectrum) {
    }
}
