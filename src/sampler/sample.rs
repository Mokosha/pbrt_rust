use camera::CameraSample;
use integrator::SurfaceIntegrator;
use integrator::VolumeIntegrator;
use sampler::Sampler;
use scene::Scene;

#[derive(Debug, Clone)]
pub struct Sample {
    camera_sample: CameraSample,
    n1D: Vec<usize>,
    n2D: Vec<usize>,
    offset1D: Vec<usize>,
    offset2D: Vec<usize>,
    samples: Vec<f32>,
}

impl Sample {
    pub fn new(sampler: &Sampler, _surf: Option<&SurfaceIntegrator>,
               _vol: Option<&VolumeIntegrator>, scene: &Scene, idx: i32) -> Sample {
        let mut s = Sample {
            camera_sample: CameraSample::empty(),
            n1D: Vec::new(),
            n2D: Vec::new(),
            offset1D: Vec::new(),
            offset2D: Vec::new(),
            samples: Vec::new(),
        };

        if let Some(vol) = _vol {
            vol.request_samples(sampler, &mut s, scene);
        }

        if let Some(surf) = _surf {
            surf.request_samples(sampler, &mut s, scene);
        }

        // Allocate sample memory
        let num_1D_samples = match s.offset1D.last() {
            None => 0,
            Some(x) => {
                assert!(s.n1D.len() > 0);
                x + *(s.n1D.last().unwrap())
            }
        };

        for x in s.offset2D.iter_mut() {
            *x += num_1D_samples
        }

        let total_num_samples = match s.offset2D.last() {
            None => num_1D_samples,
            Some(x) => {
                assert!(s.n2D.len() > 0);
                x + num_1D_samples + *(s.n2D.last().unwrap()) * 2
            }
        };

        s.samples = vec![0.0; total_num_samples];
        s
    }

    pub fn add_1d(&mut self, num: usize) -> usize {
        let first = self.n1D.is_empty();

        self.n1D.push(num);
        if first {
            self.offset1D.push(0)
        } else {
            assert!(self.offset1D.len() > 0);
            let offset_to_last = *(self.offset1D.last().unwrap());
            self.offset1D.push(offset_to_last + num);
        }

        self.n1D.len() - 1
    }

    pub fn add_2d(&mut self, num: usize) -> usize {
        let first = self.n2D.is_empty();

        self.n2D.push(num);
        if first {
            self.offset2D.push(0)
        } else {
            assert!(self.offset2D.len() > 0);
            let offset_to_last = *(self.offset2D.last().unwrap());
            self.offset2D.push(offset_to_last + (2 * num));
        }

        self.n2D.len() - 1
    }

    pub fn to_camera_sample(self) -> CameraSample { self.camera_sample }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_be_created() {
        unimplemented!()
    }

    // !FIXME! These should all also make sure that the proper amount
    // of data was allocated to store the resulting samples.

    #[ignore]
    #[test]
    fn it_can_add_1D_samples() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_add_2D_samples() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_add_both_1D_and_2D_samples() {
        unimplemented!()
    }
}
