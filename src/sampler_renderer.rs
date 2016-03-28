extern crate scoped_threadpool;
extern crate num_cpus;

use camera::Camera;
use integrator::VolumeIntegrator;
use integrator::SurfaceIntegrator;
use intersection::Intersection;
use intersection::Intersectable;
use light::Light;
use ray::RayDifferential;
use rng::RNG;
use renderer::Renderer;
use sampler::sample::Sample;
use sampler::Sampler;
use scene::Scene;
use scoped_threadpool::Pool;
use spectrum::Spectrum;

use std::cmp::max;
use std::ops::BitAnd;
use std::iter::Iterator;
use std::sync::{RwLock, Arc};

#[derive(Debug, Clone)]
pub struct SamplerRenderer {
    sampler: Sampler,
    camera: Camera,
    surface_integrator: SurfaceIntegrator,
    volume_integrator: VolumeIntegrator,

    num_tasks: usize,
    // SamplerRenderer Private Data
}

impl SamplerRenderer {
    pub fn new(sampler: Sampler, cam: Camera,
               surf: SurfaceIntegrator, vol: VolumeIntegrator) -> Self {
        let num_cpus = num_cpus::get() as u32;
        let num_pixels = cam.film().num_pixels() as u32;
        let tasks_fn = |x: u32| {
            31 - x.leading_zeros() + (if 0 == x.bitand(x - 1) { 0 } else { 1 })
        };
        let tasks = tasks_fn(max(32 * num_cpus, num_pixels / 256));

        SamplerRenderer {
            sampler: sampler,
            camera: cam,
            surface_integrator: surf,
            volume_integrator: vol,

            num_tasks: tasks as usize
        }
    }

    pub fn empty() -> SamplerRenderer {
        unimplemented!()
    }
}

fn run_task<'a>(scene: &'a Scene,
                renderer : Arc<RwLock<&'a mut SamplerRenderer>>,
                task_idx: usize, num_tasks: usize) {
    // Get sub-sampler for SamplerRendererTask
    let mut sampler = {
        if let Some(s) = renderer.read().unwrap()
            .sampler.get_sub_sampler(task_idx, num_tasks)
        { s } else { return }
    };

    // Declare local variables used for rendering loop
    let mut rng = RNG::new(task_idx);

    // Allocate space for samples and intersections
    let max_samples = sampler.maximum_sample_count() as usize;
    let mut samples : Vec<Sample> = vec![Sample::empty(); max_samples];
    let mut rays : Vec<RayDifferential> = Vec::with_capacity(max_samples);
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
                renderer.read().unwrap().camera.generate_ray_differential(&cs);

            ray.scale_differentials(1.0f32 / sampler.samples_per_pixel().sqrt());

            // Evaluate radiance along camera ray
            if ray_weight > 0f32 {
                // !FIXME! I think this synchronization is a bit too coarse grained
                let (mut ls, isect, ts) =
                    renderer.read().unwrap().li(scene, &ray, &samples[i], &mut rng);
                ls = ls * ray_weight;

                if !ls.has_nans() { panic!("Invalid radiance value!"); }
                l_s.push(ls);

                // !FIXME! I think there are times when we don't generate
                // transmissive values, and these times we shouldn't add them
                // to the list...
                t_s.push(ts);
                
                if let Some(isect_val) = isect {
                    isects.push(isect_val);
                } else {
                    // Empty intersection
                    // isects.push(Intersection::new());
                }
            }
            // else {
            //   l_s.push(Spectrum::from_value(0f32));
            //   t_s.push(Spectrum::from_value(0f32));
            //   // Empty intersection
            //   isects.push(Intersection::new());
            // }
        }

        // Report sample results to Sampler, add contributions to image
        if sampler.report_results(&mut samples, &rays, &l_s, &isects, sample_count) {
            for i in 0..sample_count {
                // !FIXME! This synchronization is still a bit coarse grained, but
                // we may be able to move the lock within a few levels to get finer
                // synchronization. Writing the computed sample is significantly
                // cheaper than the render step, though. Once we figure out a good
                // way to do the synchronization here, we should fix the atomicity of
                // adding samples to pixels in src/camera/film.rs
                let cs = samples[i].clone().to_camera_sample();
                renderer.write().unwrap()
                    .camera
                    .film_mut()
                    .add_sample(&cs, &l_s[i]);
            }
        }
    }
}

impl Renderer for SamplerRenderer {
    fn render(&mut self, scene : &Scene) {
        // Allow integrators to do preprocessing for the scene
        self.surface_integrator.preprocess(scene, &(self.camera));
        self.volume_integrator.preprocess(scene, &(self.camera));

        // Allocate and initialize sample
        let num_tasks = self.num_tasks;

        // Create and launch SampleRendererTasks for rendering image
        {
            let num_cpus = num_cpus::get();
            let num_pixels = self.camera.film().num_pixels();

            let task_data_shared = Arc::new(RwLock::new(self));

            println!("Running {:?} tasks on pool with {} cpus",
                     num_tasks, num_cpus);

            Pool::new(num_cpus as u32).scoped(|scope| {
                for i in 0..num_tasks {
                    let rend = task_data_shared.clone();
                    scope.execute(move || run_task(scene, rend, i, num_tasks));
                }
            });
        }

        // Clean up after rendering and store final image    
    }

    fn li<'a>(&self, scene: &'a Scene, ray: &RayDifferential,
              sample: &Sample,
              rng: &mut RNG) -> (Spectrum, Option<Intersection>, Spectrum) {

        // Allocate variables for isect and T if needed
        let (isect, li) =
            if let Some(mut scene_isect) = scene.intersect(&ray.ray) {
                let l = self.surface_integrator.li(scene, self, ray,
                                                   &mut scene_isect, sample, rng);
                (Some(scene_isect), l)
            } else {
                // Handle ray that doesn't intersect any geometry
                let zero_spect = Spectrum::from_value(0f32);
                let accum = |acc, light: &Light| acc + light.le(ray);
                (None, scene.lights().iter().fold(zero_spect, accum))
            };

        let mut local_trans = Spectrum::from_value(0f32);
        let lvi = self.volume_integrator.li(scene, self, ray, sample,
                                            rng, &mut local_trans);

        (local_trans * li + lvi, isect, local_trans)
    }

    fn transmittance(&self, scene: &Scene, ray: &RayDifferential,
                     sample: &Sample, rng: &mut RNG) -> Spectrum {
        let mut local_trans = Spectrum::from_value(0f32);
        self.volume_integrator.li(scene, self, ray, sample, rng, &mut local_trans)
    }

    // Rnderer Interface
}
