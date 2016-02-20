use camera::CameraSample;
use integrator::SurfaceIntegrator;
use integrator::VolumeIntegrator;
use intersection::Intersection;
use ray::RayDifferential;
use rng::RNG;
use spectrum::Spectrum;
use scene::Scene;
use utils::Lerp;

#[derive(Debug, Clone)]
pub struct Sample {
    camera_sample: CameraSample,
}

impl Sample {
    pub fn new<Surf : SurfaceIntegrator, Vol : VolumeIntegrator>(
        sampler: &Sampler, surf: &Surf, vol: &Vol, scene: &Scene, idx: i32)
        -> Sample { unimplemented!() }

    pub fn to_camera_sample(self) -> CameraSample { self.camera_sample }
}

#[derive(Debug, Clone)]
pub struct SamplerBase {
    x_pixel_start: i32,
    x_pixel_end: i32,
    y_pixel_start: i32,
    y_pixel_end: i32,
    samples_per_pixel: usize,
    shutter_open: f32,
    shutter_close: f32
}

pub struct Sampler {
    base: SamplerBase
}

impl Sampler {
    pub fn new() -> Sampler { unimplemented!() }

    fn base(&self) -> &SamplerBase { &self.base }

    pub fn get_sub_sampler(&self, task_idx : i32, num_tasks : i32)
                           -> Option<Sampler> {
        unimplemented!()
    }

    pub fn maximum_sample_count(&self) -> i32 {
        unimplemented!()
    }

    pub fn get_more_samples<T: RNG>(&mut self, samples: &mut Vec<Sample>, rng: &mut T) {
        unimplemented!()
    }

    pub fn samples_per_pixel(&self) -> f32 {
        unimplemented!()
    }

    pub fn report_results(&self, samples: &Vec<Sample>,
                          rays: &Vec<RayDifferential>,
                          ls: &Vec<Spectrum>,
                          isects: &Vec<Intersection>,
                          sample_count: usize) -> bool { true }

    pub fn compute_sub_window(&self, num: usize, count: usize) -> (i32, i32, i32, i32) {
        // Determine how many tiles to use in each dimension nx and ny
        let dx = (self.base().x_pixel_end - self.base().x_pixel_start) as usize;
        let dy = (self.base().y_pixel_end - self.base().y_pixel_start) as usize;

        let mut nx = count;
        let mut ny = 1;
        while (nx % 2) == 0 && 2*dx*ny < dy*nx {
            nx /= 2;
            ny *= 2;
        }

        // Compute x and y pixel sample range for sub window
        let xo = num % nx;
        let yo = num / nx;

        let tx0 = (xo as f32) / (nx as f32);
        let tx1 = ((xo + 1) as f32) / (nx as f32);

        let ty0 = (yo as f32) / (ny as f32);
        let ty1 = ((yo + 1) as f32) / (ny as f32);

        let new_x_start = (self.base().x_pixel_start as f32).lerp(&(self.base().x_pixel_end as f32), tx0) as i32;
        let new_x_end = (self.base().x_pixel_start as f32).lerp(&(self.base().x_pixel_end as f32), tx1) as i32;
        let new_y_start = (self.base().y_pixel_start as f32).lerp(&(self.base().y_pixel_end as f32), tx0) as i32;
        let new_y_end = (self.base().y_pixel_start as f32).lerp(&(self.base().y_pixel_end as f32), tx1) as i32;

        (new_x_start, new_x_end, new_y_start, new_y_end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_be_created() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_tile_windows() {
        unimplemented!()
    }
}
