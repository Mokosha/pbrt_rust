extern crate scoped_threadpool;
extern crate num_cpus;

use camera::Camera;
use camera::CameraSample;
use camera::film::Film;
use integrator::Integrator;
use integrator::VolumeIntegrator;
use integrator::SurfaceIntegrator;
use intersection::Intersection;
use intersection::Intersectable;
use ray;
use rng::RNG;
use rng::PseudoRNG;
use renderer::Renderer;
use sampler::sample::Sample;
use sampler::Sampler;
use scene;
use scoped_threadpool::Pool;
use spectrum::Spectrum;
use transform::transform::Transform;
use transform::animated::AnimatedTransform;

use std::ops::BitAnd;
use std::iter::Iterator;
use std::sync::{RwLock, Arc};

pub struct SamplerRenderer {
    sampler: Sampler,
    camera: Camera,
    surface_integrator: SurfaceIntegrator,
    volume_integrator: VolumeIntegrator,
    // SamplerRenderer Private Data
}

impl SamplerRenderer {
    pub fn new(
        sampler: Sampler, cam: Camera,
        surf: SurfaceIntegrator,
        vol: VolumeIntegrator) -> Self {
        SamplerRenderer {
            sampler: sampler,
            camera: cam,
            surface_integrator: surf,
            volume_integrator: vol,
        }
    }

    pub fn empty() -> SamplerRenderer {
        unimplemented!()
    }
}

struct SamplerRendererTaskData<'a, 'b> {
    scene: &'a scene::Scene,
    renderer: &'a mut SamplerRenderer,
    sample: &'b mut Sample
}

impl<'a, 'b>
    SamplerRendererTaskData<'a, 'b> {
        fn new(scene: &'a scene::Scene,
               renderer: &'a mut SamplerRenderer,
               sample: &'b mut Sample) ->
            SamplerRendererTaskData<'a, 'b> {
                SamplerRendererTaskData {
                    scene: scene,
                    renderer: renderer,
                    sample: sample
                }
            }
    }

fn run_task<'a, 'b>(data : Arc<RwLock<SamplerRendererTaskData<'a, 'b>>>, task_idx: i32, num_tasks: i32) {
    // Get sub-sampler for SamplerRendererTask
    let mut sampler = {
        if let Some(s) = data.read().unwrap().renderer.sampler.get_sub_sampler(task_idx, num_tasks)
        { s } else { return }
    };
    
    let scene = data.read().unwrap().scene;

    // Declare local variables used for rendering loop
    let mut rng = PseudoRNG::new(task_idx);
    
    // Allocate space for samples and intersections
    let max_samples = sampler.maximum_sample_count() as usize;
    let mut samples : Vec<Sample> = (0..max_samples).map(|_| data.read().unwrap().sample.clone()).collect();
    let mut rays : Vec<ray::RayDifferential> = Vec::with_capacity(max_samples);
    let mut l_s : Vec<Spectrum> = Vec::with_capacity(max_samples);
    let mut t_s : Vec<Spectrum> = Vec::with_capacity(max_samples);
    let mut isects : Vec<Intersection> = Vec::with_capacity(max_samples);

    // Get samples from Sampler and update image
    loop {
        sampler.get_more_samples(&mut samples, &mut rng);
        let sample_count = samples.len();
        if sample_count == 0 { break; }

        // Generate camera rays and compute radiance along rays
        for i in 0..sample_count {
            // Find camera ray for sample[i]
            let cs = samples[i].clone().to_camera_sample();
            let (ray_weight, mut ray) =
                data.read().unwrap().renderer.camera.generate_ray_differential(&cs);

            ray.scale_differentials(1.0f32 / sampler.samples_per_pixel().sqrt());

            // Evaluate radiance along camera ray
            if ray_weight > 0f32 {
                // !FIXME! I think this synchronization is a bit too coarse grained
                let (mut ls, isect, ts) = data.read().unwrap().renderer.li(scene, &ray, &(samples[i]), &mut rng);
                ls = ls * ray_weight;

                if !ls.has_nans() { panic!("Invalid radiance value!"); }
                l_s.push(ls);

                // !FIXME! I think there are times when we don't generate transmissive
                // values, and these times we shouldn't add them to the list...
                t_s.push(ts);
                
                if let Some(isect_val) = isect {
                    isects.push(isect_val);
                } else {
                    // Empty intersection
                    // isects.push(Intersection::new());
                }
            } else {
                l_s.push(Spectrum::from_value(0f32));
                t_s.push(Spectrum::from_value(0f32));
                // Empty intersection
                // isects.push(Intersection::new());
            }
        }

        // Report sample results to Sampler, add contributions to image
        if sampler.report_results(&samples, &rays, &l_s, &isects, sample_count) {
            for i in 0..sample_count {
                // !FIXME! This synchronization is still a bit coarse grained, but
                // we may be able to move the lock within a few levels to get finer
                // synchronization. Writing the computed sample is significantly
                // cheaper than the render step, though
                let cs = samples[i].clone().to_camera_sample();
                data.write().unwrap().renderer.camera.film_mut().add_sample(&cs, &l_s[i]);
            }
        }

        samples.clear();
        rays.clear();
        l_s.clear();
        t_s.clear();
        isects.clear();
    }
}

impl Renderer for SamplerRenderer {
    fn render(&mut self, scene : &scene::Scene) {
        // Allow integrators to do preprocessing for the scene
        self.surface_integrator.preprocess(scene, &(self.camera));
        self.volume_integrator.preprocess(scene, &(self.camera));

        // Allocate and initialize sample
        let mut sample = Sample::new(&(self.sampler), Some(&self.surface_integrator),
                                     Some(&self.volume_integrator), &scene, 1);

        // Create and launch SampleRendererTasks for rendering image
        {
            let num_cpus = num_cpus::get() as i32;
            let num_pixels = self.camera.film().num_pixels();

            let num_tasks = (|x : i32| {
                31 - (x.leading_zeros() as i32) + (if 0 == x.bitand(x - 1) { 0 } else { 1 })
            }) (::std::cmp::max(32 * num_cpus, num_pixels / (16 * 16)));

            let task_data = SamplerRendererTaskData::new(scene, self, &mut sample);
            let task_data_shared = Arc::new(RwLock::new(task_data));

            println!("Running {} tasks on pool with {} cpus", num_tasks, num_cpus);
            Pool::new(num_cpus as u32).scoped(|scope| {
                for i in 0..num_tasks {
                    let data = task_data_shared.clone();
                    scope.execute(move || run_task(data, i, num_tasks));
                }
            });
        }

        // Clean up after rendering and store final image    
    }

    fn li<'a, T:RNG>(
        &self, scene: &'a scene::Scene, ray: &ray::RayDifferential,
        sample: &Sample, rng: &mut T) -> (Spectrum, Option<Intersection>, Spectrum) {
        // Allocate variables for isect and T if needed
        let (isect, li) =
            if let Some(mut scene_isect) = scene.intersect(&ray.ray) {
                let l = self.surface_integrator.li(scene, self, ray, &mut scene_isect, sample, rng);
                (Some(scene_isect), l)
            } else {
                // Handle ray that doesn't intersect any geometry
                (None, scene.lights().iter().fold(Spectrum::from_value(0f32), |acc, light| acc + light.le(ray)))
            };

        let mut local_trans = Spectrum::from_value(0f32);
        let lvi = self.volume_integrator.li(scene, self, ray, sample, rng, &mut local_trans);

        (local_trans * li + lvi, isect, local_trans)
    }

    fn transmittance<T:RNG>(
        &self, scene: &scene::Scene, ray: &ray::RayDifferential,
        sample: &Sample, rng: &mut T) -> Spectrum {
        let mut local_trans = Spectrum::from_value(0f32);
        self.volume_integrator.li(scene, self, ray, sample, rng, &mut local_trans)
    }

    // Rnderer Interface
}
