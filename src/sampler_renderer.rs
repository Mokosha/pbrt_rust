extern crate num_cpus;

use camera;
use integrator;
use integrator::Integrator;
use ray;
use renderer;
use sampler;
use scene;
use intersection;

use std::ops::BitAnd;

pub struct SamplerRenderer {
    sampler : sampler::Sampler,
    camera : camera::Camera,
    surface_integrator : integrator::SurfaceIntegrator,
    volume_integrator : integrator::VolumeIntegrator,
    // SamplerRenderer Private Data
}

impl SamplerRenderer {
    pub fn new(sampler : sampler::Sampler, cam : camera::Camera,
               surf: integrator::SurfaceIntegrator,
               vol: integrator::VolumeIntegrator) -> SamplerRenderer {
        SamplerRenderer {
            sampler: sampler,
            camera: cam,
            surface_integrator: surf,
            volume_integrator: vol
        }
    }
}

struct SamplerRendererTask;

impl SamplerRendererTask {
    fn new(scene: &scene::Scene,
           renderer: &SamplerRenderer,
           camera: &camera::Camera,
           sampler: &sampler::Sampler,
           sample: &sampler::Sample,
           task_idx: i32,
           num_tasks: i32) -> SamplerRendererTask { SamplerRendererTask }
}

impl renderer::Renderer for SamplerRenderer {
    fn render(&mut self, scene : &scene::Scene) {
        // Allow integrators to do preprocessing for the scene
        self.surface_integrator.preprocess(scene, &(self.camera));
        self.volume_integrator.preprocess(scene, &(self.camera));

        // Allocate and initialize sample
        let sample = sampler::Sample::new(&(self.sampler), &(self.surface_integrator),
                                          &(self.volume_integrator), &scene);

        // Create and launch SampleRendererTasks for rendering image
        {
            let num_pixels = self.camera.film().num_pixels();

            let num_tasks = (|x : i32| {
                32 - (x.leading_zeros() as i32) + (if 0 == x.bitand(x - 1) { 0 } else { 1 })
            }) (::std::cmp::max(32 * (num_cpus::get() as i32), num_pixels / (16 * 16)));

            let mut render_tasks = vec![];
            for i in 0..num_tasks {
                let task = SamplerRendererTask::new(&scene, &self, &self.camera,
                                                    &self.sampler, &sample, num_tasks - 1 - i,
                                                    num_tasks);
                render_tasks.push(::std::thread::spawn(move || { }));
            }

            for task in render_tasks {
                let _ = task.join();
            }
        }

        // Clean up after rendering and store final image
    
    }

    fn li<T:renderer::RNG>(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &sampler::Sample, rng: &mut T,
        isect: &mut Option<intersection::Intersection>,
        spect: &mut Option<renderer::Spectrum>) {
    }

    fn transmittance<T:renderer::RNG>(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &sampler::Sample, rng: &mut T) {
    }

    // Rnderer Interface
}
