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

struct SamplerRendererTaskData<'a, 'b> {
    scene: &'a scene::Scene,
    renderer: &'a mut SamplerRenderer,
    sample: &'b mut sampler::Sample
}

impl<'a, 'b> SamplerRendererTaskData<'a, 'b> {
    fn new(scene: &'a scene::Scene,
           renderer: &'a mut SamplerRenderer,
           sample: &'b mut sampler::Sample) ->
        SamplerRendererTaskData<'a, 'b> {
            SamplerRendererTaskData {
                scene: scene,
                renderer: renderer,
                sample: sample
            }
        }

    fn get_camera(&'a mut self) -> &'a mut camera::Camera { &mut self.renderer.camera }
    fn get_sampler(&'a mut self) -> &'a mut sampler::Sampler { &mut self.renderer.sampler }
}

struct SamplerRendererTask<'a, 'b> {
    // SamplerRenderTask private data
    data : ::std::sync::Arc<::std::sync::Mutex<SamplerRendererTaskData<'a, 'b>>>,
    task_idx : i32,
    num_tasks : i32
}

fn run_task(task : SamplerRendererTask) {
    // Get sub-sampler for SamplerRendererTask
    // Declare local variables used for rendering loop
    // Allocate space for samples and intersections
    // Get samples from Sampler and update image
    // Clean up after SamplerRendererTask is done with its image region
}

impl renderer::Renderer for SamplerRenderer {
    fn render(&mut self, scene : &scene::Scene) {
        // Allow integrators to do preprocessing for the scene
        self.surface_integrator.preprocess(scene, &(self.camera));
        self.volume_integrator.preprocess(scene, &(self.camera));

        // Allocate and initialize sample
        let mut sample = sampler::Sample::new(&(self.sampler), &(self.surface_integrator),
                                              &(self.volume_integrator), &scene);

        // Create and launch SampleRendererTasks for rendering image
        {
            let num_cpus = num_cpus::get() as i32;
            let num_pixels = self.camera.film().num_pixels();

            let num_tasks = (|x : i32| {
                31 - (x.leading_zeros() as i32) + (if 0 == x.bitand(x - 1) { 0 } else { 1 })
            }) (::std::cmp::max(32 * num_cpus, num_pixels / (16 * 16)));

            let task_data =
                ::std::sync::Arc::new(
                ::std::sync::Mutex::new(
                    SamplerRendererTaskData::new(&scene, self, &mut sample)));

            Pool::new(num_cpus as u32).scoped(|scope| {
                (0..num_tasks).map(|i| {
                    let task = SamplerRendererTask {
                        data: task_data.clone(),
                        task_idx: i,
                        num_tasks: num_tasks
                    };

                    unsafe {
                        scope.execute(move || run_task(task));
                    }
                });
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
