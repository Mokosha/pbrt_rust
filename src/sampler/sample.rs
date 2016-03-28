use camera::CameraSample;
use integrator::SurfaceIntegrator;
use integrator::VolumeIntegrator;
use sampler::Sampler;
use scene::Scene;

#[derive(Debug, Clone)]
pub struct Sample {
    pub camera_sample: CameraSample,
    pub num_1d: Vec<usize>,
    pub num_2d: Vec<usize>,
    pub offset_1d: Vec<usize>,
    pub offset_2d: Vec<usize>,
    pub samples: Vec<f32>,
}

impl Sample {
    pub fn empty() -> Sample {
        Sample {
            camera_sample: CameraSample::empty(),
            num_1d: Vec::new(),
            num_2d: Vec::new(),
            offset_1d: Vec::new(),
            offset_2d: Vec::new(),
            samples: Vec::new(),            
        }
    }

    pub fn new(sampler: &Sampler, _surf: Option<&SurfaceIntegrator>,
               _vol: Option<&VolumeIntegrator>, scene: &Scene) -> Sample {
        let mut s = Sample::empty();
        if let Some(vol) = _vol {
            vol.request_samples(sampler, &mut s, scene);
        }

        if let Some(surf) = _surf {
            surf.request_samples(sampler, &mut s, scene);
        }

        // Allocate sample memory
        let num_1d_samples = match s.offset_1d.last() {
            None => 0,
            Some(x) => {
                assert!(s.num_1d.len() > 0);
                x + *(s.num_1d.last().unwrap())
            }
        };

        for x in s.offset_2d.iter_mut() {
            *x += num_1d_samples
        }

        let total_num_samples = match s.offset_2d.last() {
            None => num_1d_samples,
            Some(x) => {
                assert!(s.num_2d.len() > 0);
                x + num_1d_samples + *(s.num_2d.last().unwrap()) * 2
            }
        };

        s.samples = vec![0.0; total_num_samples];
        s
    }

    pub fn add_1d(&mut self, num: usize) -> usize {
        let first = self.num_1d.is_empty();

        self.num_1d.push(num);
        if first {
            self.offset_1d.push(0)
        } else {
            assert!(self.offset_1d.len() > 0);
            let offset_to_last = *(self.offset_1d.last().unwrap());
            self.offset_1d.push(offset_to_last + num);
        }

        self.num_1d.len() - 1
    }

    pub fn add_2d(&mut self, num: usize) -> usize {
        let first = self.num_2d.is_empty();

        self.num_2d.push(num);
        if first {
            self.offset_2d.push(0)
        } else {
            assert!(self.offset_2d.len() > 0);
            let offset_to_last = *(self.offset_2d.last().unwrap());
            self.offset_2d.push(offset_to_last + (2 * num));
        }

        self.num_2d.len() - 1
    }

    pub fn to_camera_sample(self) -> CameraSample { self.camera_sample }
}

#[cfg(test)]
mod tests {

    // !FIXME! These should all also make sure that the proper amount
    // of data was allocated to store the resulting samples. These
    // can't be properly tested until we get integrators working...

    #[ignore]
    #[test]
    fn it_can_be_created() {
    }

    #[ignore]
    #[test]
    fn it_can_add_1d_samples() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_add_2d_samples() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_add_both_1d_and_2d_samples() {
        unimplemented!()
    }
}
