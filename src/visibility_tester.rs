use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use sampler::sample::Sample;
use scene::Scene;
use spectrum::Spectrum;
use ray::Ray;
use ray::RayDifferential;
use renderer::Renderer;
use rng::RNG;

#[derive(Debug, PartialEq, Clone)]
pub struct VisibilityTester(Ray);

impl VisibilityTester {
    pub fn segment(p1: Point, eps1: f32, p2: Point, eps2: f32, time: f32)
                   -> VisibilityTester {
        let dist = p1.distance(&p2);
        let dir = (p2 - &p1) / dist;
        let mut r = Ray::new_with(p1, dir, eps1);
        r.set_maxt((1.0 - eps2) * dist);
        r.set_time(time);
        VisibilityTester(r)
    }

    pub fn ray(p: Point, eps: f32, w: Vector, time: f32) -> VisibilityTester {
        let mut r = Ray::new_with(p, w, eps);
        r.set_time(time);
        VisibilityTester(r)
    }

    pub fn unoccluded(&self, scene: &Scene) -> bool {
        ! scene.intersect_p(&self.0)
    }

    pub fn transmittance<R: Renderer>(
        &self, scene: &Scene, renderer: &R,
        sample: &Sample, rng: &mut RNG) -> Spectrum {
        let rd = RayDifferential::from(self.0.clone());
        renderer.transmittance(scene, &rd, sample, rng)
    }
}
