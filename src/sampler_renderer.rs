extern crate scoped_threadpool;
extern crate num_cpus;

use camera;
use integrator;
use integrator::Integrator;
use ray;
use renderer;
use sampler;
use scene;
use scoped_threadpool::Pool;
use intersection;

use std::ops::BitAnd;
use std::sync::{Mutex, MutexGuard, Arc};

pub struct SamplerRenderer {
    sampler: sampler::Sampler,
    camera: camera::Camera,
    surface_integrator: integrator::SurfaceIntegrator,
    volume_integrator: integrator::VolumeIntegrator,
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

    pub fn new_empty() -> SamplerRenderer {
        SamplerRenderer {
            sampler: sampler::Sampler,
            camera: camera::Camera::new(512, 512),
            surface_integrator: integrator::SurfaceIntegrator,
            volume_integrator: integrator::VolumeIntegrator,
        }
    }
}

struct SamplerRendererTaskData<'a> {
    scene: &'a scene::Scene,
    renderer: &'a mut SamplerRenderer,
    sample: &'a mut sampler::Sample
}

impl<'a> SamplerRendererTaskData<'a> {
    fn new(scene: &'a scene::Scene,
           renderer: &'a mut SamplerRenderer,
           sample: &'a mut sampler::Sample) ->
        SamplerRendererTaskData<'a> {
            SamplerRendererTaskData {
                scene: scene,
                renderer: renderer,
                sample: sample
            }
        }

    fn get_camera(&mut self) -> &mut camera::Camera { &mut self.renderer.camera }
    fn get_sampler(&mut self) -> &mut sampler::Sampler { &mut self.renderer.sampler }
}

fn run_task<'a, 'b>(task_data : &'b Arc<Mutex<&'a mut SamplerRendererTaskData<'a>>>,
            task_idx: i32, num_tasks: i32) {
    // Get sub-sampler for SamplerRendererTask
    let sampler = {
        let mut data : MutexGuard<'b, &'a mut SamplerRendererTaskData<'a>> =
            task_data.lock().unwrap();
        if let Some(s) = data.get_sampler().get_sub_sampler(task_idx, num_tasks)
        { s } else { return }
    };
    
    // Declare local variables used for rendering loop
    let rng = renderer::PseudoRNG::new(task_idx);
    
    // Allocate space for samples and intersections
    let max_samples = sampler.maximum_sample_count() as usize;
    let samples : Vec<sampler::Sample> = {
        let mut data : MutexGuard<'b, &'a mut SamplerRendererTaskData<'a>> =
            task_data.lock().unwrap();
        (0..max_samples).map(|_| data.sample.clone()).collect()
    };
    let rays : Vec<ray::RayDifferential> = Vec::with_capacity(max_samples);
    let l_s : Vec<renderer::Spectrum> = Vec::with_capacity(max_samples);
    let t_s : Vec<renderer::Spectrum> = Vec::with_capacity(max_samples);
    let isects : Vec<intersection::Intersection> = Vec::with_capacity(max_samples);

    // Get samples from Sampler and update image
    // Clean up after SamplerRendererTask is done with its image region

    // !DEBUG!
    let sample = {
        let mut data : MutexGuard<'b, &'a mut SamplerRendererTaskData<'a>> = task_data.lock().unwrap();
        data.sample.idx
    };
    println!("Got sample {} fo task {} of {}", sample, task_idx, num_tasks);
}

impl renderer::Renderer for SamplerRenderer {
    fn render(&mut self, scene : &scene::Scene) {
        // Allow integrators to do preprocessing for the scene
        self.surface_integrator.preprocess(scene, &(self.camera));
        self.volume_integrator.preprocess(scene, &(self.camera));

        // Allocate and initialize sample
        let mut sample = sampler::Sample::new(&(self.sampler), &(self.surface_integrator),
                                              &(self.volume_integrator), &scene, 1);

        // Create and launch SampleRendererTasks for rendering image
        {
            let num_cpus = num_cpus::get() as i32;
            let num_pixels = self.camera.film().num_pixels();

            let num_tasks = (|x : i32| {
                31 - (x.leading_zeros() as i32) + (if 0 == x.bitand(x - 1) { 0 } else { 1 })
            }) (::std::cmp::max(32 * num_cpus, num_pixels / (16 * 16)));

            let mut task_data = SamplerRendererTaskData::new(scene, self, &mut sample);
            let task_data_async = Arc::new(Mutex::new(&mut task_data));

            println!("Running {} tasks on pool with {} cpus", num_tasks, num_cpus);
            Pool::new(num_cpus as u32).scoped(|scope| {
                let _ : Vec<_> = (0..num_tasks).map(|i| {
                    let data = task_data_async.clone();
                    unsafe { scope.execute(move || run_task(&data, i, num_tasks)); }
                }).collect();
            });
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
