use camera::CameraSample;
use spectrum::Spectrum;

#[derive(Debug, Clone)]
pub struct Film {
    x_resolution : usize,
    y_resolution : usize
}

impl Film {
    pub fn new(xres: usize, yres: usize) -> Film {
        Film { x_resolution: xres, y_resolution: yres }
    }

    pub fn x_res(&self) -> usize { self.x_resolution }
    pub fn y_res(&self) -> usize { self.y_resolution }
    pub fn num_pixels(&self) -> usize { self.x_res() * self.y_res() }
    pub fn add_sample(&mut self, sample: &CameraSample, ls: &Spectrum) {
    }
}
