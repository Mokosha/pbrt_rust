use camera::CameraSample;
use rng::RNG;
use sampler::base::SamplerBase;
use sampler::sample::Sample;
use utils::Lerp;

use montecarlo::latin_hypercube;
use montecarlo::stratified_sample_1d;
use montecarlo::stratified_sample_2d;

#[derive(Debug, Clone, PartialEq)]
pub struct StratifiedSampler {
    base: SamplerBase,
    x_pixel_samples: usize,
    y_pixel_samples: usize,
    jitter_samples: bool,
    x_pos: i32,
    y_pos: i32,
    sample_buf: Vec<f32>
}

impl StratifiedSampler {
    pub fn new(x_start: i32, x_end: i32, y_start: i32, y_end: i32,
               xs: usize, ys: usize, jitter: bool,
               sopen: f32, sclose: f32) -> StratifiedSampler {
        let b = SamplerBase::new(x_start, x_end, y_start, y_end,
                                 xs * ys, sopen, sclose);
        StratifiedSampler {
            base: b.clone(),
            x_pixel_samples: xs,
            y_pixel_samples: ys,
            jitter_samples: jitter,
            x_pos: b.x_pixel_start,
            y_pos: b.y_pixel_start,
            sample_buf: vec![0.0; 5 * (xs * ys as usize)]
        }
    }

    pub fn base(&self) -> &SamplerBase { &self.base }

    pub fn maximum_sample_count(&self) -> usize {
        self.x_pixel_samples * self.y_pixel_samples
    }

    pub fn get_sub_sampler(&self, num: usize,
                           count: usize) -> Option<StratifiedSampler> {
        let (x0, x1, y0, y1) = self.base.compute_sub_window(num, count);
        if x0 == x1 || y0 == y1 {
            None
        } else {
            Some(StratifiedSampler::new(x0, x1, y0, y1,
                                        self.x_pixel_samples,
                                        self.y_pixel_samples,
                                        self.jitter_samples,
                                        self.base.shutter_open,
                                        self.base.shutter_close))
        }
    }

    pub fn get_more_samples(&mut self, samples: &mut Vec<Sample>,
                            rng: &mut RNG) -> usize {
        if self.y_pos == self.base.y_pixel_end { return 0; }

        let num_samples = self.x_pixel_samples * self.y_pixel_samples;

        // Generate stratified samples for (xpos, ypos)...
        // Generate initial stratified samples into sampleBuf memory
        let (mut image_samples, mut not_image_samples) =
            self.sample_buf.split_at_mut(2 * num_samples);
        let (mut lens_samples, mut time_samples) =
            not_image_samples.split_at_mut(2 * num_samples);
        stratified_sample_2d(image_samples, self.x_pixel_samples,
                             self.y_pixel_samples, rng, self.jitter_samples);
        stratified_sample_2d(lens_samples, self.x_pixel_samples,
                             self.y_pixel_samples, rng, self.jitter_samples);
        stratified_sample_1d(time_samples, num_samples,
                             rng, self.jitter_samples);

        // Shift stratified image samples to pixel coordinates
        for i in 0..num_samples {
            image_samples[2 * i + 0] += self.x_pos as f32;
            image_samples[2 * i + 1] += self.y_pos as f32;
        }

        // Decorrelate sample dimensions
        rng.shuffle(&mut lens_samples, 2);
        rng.shuffle(&mut time_samples, 1);

        // Initialize stratified samples with sample values
        for i in 0..num_samples {
            let t = self.base.shutter_open.lerp(
                &self.base.shutter_close, time_samples[i]);

            samples[i].camera_sample = CameraSample::new(
                image_samples[2*i + 0],
                image_samples[2*i + 1],
                lens_samples[2*i + 0],
                lens_samples[2*i + 1],
                t);

            // Generate stratified samples for integrators
            let sz_and_off_1d: Vec<(usize, usize)> = samples[i].num_1d.iter().zip(
                samples[i].offset_1d.iter()).map(|(x, y)| (*x, *y)).collect();

            for (num, off) in sz_and_off_1d {
                let (oned, _) = samples[i].samples.split_at_mut(off);
                latin_hypercube(oned, num, 1, rng);
            }

            let sz_and_off_2d: Vec<(usize, usize)> = samples[i].num_2d.iter().zip(
                samples[i].offset_2d.iter()).map(|(x, y)| (*x, *y)).collect();

            for (num, off) in sz_and_off_2d {
                let (twod, _) = samples[i].samples.split_at_mut(off);
                latin_hypercube(twod, num, 2, rng);
            }
        }

        // Advance to next pixel for stratified sampling
        self.x_pos += 1;
        if self.x_pos == self.base.x_pixel_end {
            self.x_pos = self.base.x_pixel_start;
            self.y_pos += 1;
        }

        num_samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sampler::base::SamplerBase;

    #[test]
    fn it_can_be_created() {
        let s = StratifiedSampler::new(0, 10, 0, 10, 20, 5, true, 0.0, 1.0);

        assert_eq!(s.base, SamplerBase::new(0, 10, 0, 10, 100, 0.0, 1.0));
        assert_eq!(s.x_pixel_samples, 20);
        assert_eq!(s.y_pixel_samples, 5);
        assert_eq!(s.jitter_samples, true);
        assert_eq!(s.x_pos, 0);
        assert_eq!(s.y_pos, 0);
        assert_eq!(s.sample_buf, vec![0.0; 500]);
    }

    #[ignore]
    #[test]
    fn it_can_generate_stratified_samples() {
        // !FIXME! Not really sure how to test stochastic
        // methods....
        unimplemented!()
    }
}
