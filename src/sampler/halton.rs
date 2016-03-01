use camera::CameraSample;
use rng::RNG;
use sampler::base::SamplerBase;
use sampler::sample::Sample;
use utils::Lerp;

use montecarlo::radical_inverse;
use montecarlo::latin_hypercube;

#[derive(Debug, Clone, PartialEq)]
pub struct HaltonSampler {
    base: SamplerBase,
    wanted_samples: usize,
    current_sample: usize
}

impl HaltonSampler {
    fn new(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
           samples_per_pixel: usize, sopen: f32, sclose: f32) -> HaltonSampler {
        let dx = x_end - x_start;
        let dy = y_end - y_start;
        let num_samples = (|x| x * x)(if dx > dy { dx } else { dy }) as usize;
        HaltonSampler {
            base: SamplerBase::new(x_start, x_end, y_start, y_end,
                                   samples_per_pixel, sopen, sclose),
            wanted_samples: num_samples * samples_per_pixel,
            current_sample: 0
        }
    }

    pub fn base(&self) -> &SamplerBase { &self.base }

    pub fn maximum_sample_count(&self) -> usize { 1 }

    pub fn get_sub_sampler(&self, num: usize,
                           count: usize) -> Option<HaltonSampler> {
        let (x0, x1, y0, y1) = self.base.compute_sub_window(num, count);
        if x0 == x1 || y0 == y1 {
            None
        } else {
            Some(HaltonSampler::new(x0, x1, y0, y1,
                                    self.base.samples_per_pixel,
                                    self.base.shutter_open,
                                    self.base.shutter_close))
        }        
    }    

    fn get_more_samples(&mut self, samples: &mut Vec<Sample>,
                        rng: &mut RNG) -> usize {
        loop {
            let mut sample: &mut Sample = samples.get_mut(0).unwrap();

            if self.current_sample == self.wanted_samples {
                return 0;
            }

            // Generate sample with halton sequence and reject if outside image extent
            let u = radical_inverse(self.current_sample, 3) as f32;
            let v = radical_inverse(self.current_sample, 2) as f32;

            let lerp_delta = {
                let dx = (self.base.x_pixel_end - self.base.x_pixel_start) as f32;
                let dy = (self.base.y_pixel_end - self.base.y_pixel_start) as f32;
                dy.max(dx)
            };

            let image_x = (self.base.x_pixel_start as f32)
                .lerp(&((self.base.x_pixel_start as f32) + lerp_delta), u);
            let image_y = (self.base.y_pixel_start as f32)
                .lerp(&((self.base.y_pixel_start as f32) + lerp_delta), v);
            self.current_sample += 1;

            if image_x >= (self.base.x_pixel_end as f32) ||
               image_y >= (self.base.y_pixel_end as f32)
            {
                continue;
            }

            // Generate lens time and integrator samples for HaltonSampler
            let lens_u = radical_inverse(self.current_sample, 5) as f32;
            let lens_v = radical_inverse(self.current_sample, 7) as f32;
            let t = self.base.shutter_open.lerp(
                &self.base.shutter_close,
                radical_inverse(self.current_sample, 11) as f32);

            sample.camera_sample = CameraSample::new(
                image_x, image_y, lens_u, lens_v, t);

            let sz_and_off_1d: Vec<(usize, usize)> = sample.n1D.iter().zip(
                sample.offset1D.iter()).map(|(x, y)| (*x, *y)).collect();

            for (num, off) in sz_and_off_1d {
                let (oned, _) = sample.samples.split_at_mut(off);
                latin_hypercube(oned, num, 1, rng);
            }

            let sz_and_off_2d: Vec<(usize, usize)> = sample.n2D.iter().zip(
                sample.offset2D.iter()).map(|(x, y)| (*x, *y)).collect();

            for (num, off) in sz_and_off_2d {
                let (twod, _) = sample.samples.split_at_mut(off);
                latin_hypercube(twod, num, 2, rng);
            }

            return 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sampler::base::SamplerBase;

}
