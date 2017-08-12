use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use volume::VolumeRegion;

use std::f32;

#[derive(Debug)]
pub struct AggregateVolumeRegion {
    regions: Vec<Box<VolumeRegion>>,
    bound: BBox,
}

impl AggregateVolumeRegion {
    pub fn new(r: Vec<Box<VolumeRegion>>) -> AggregateVolumeRegion {
        let b = r.iter().fold(BBox::new(), |old_box, region| {
            old_box.unioned_with(region.world_bound())
        });
        AggregateVolumeRegion { regions: r, bound: b }
    }
}

impl HasBounds for AggregateVolumeRegion {
    fn world_bound(&self) -> BBox { self.bound.clone() }
}

impl Intersectable<(f32, f32)> for AggregateVolumeRegion {
    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        let (t0, t1) = self.regions.iter().fold((f32::MAX, -f32::MAX), |(t0, t1), r| {
            if let Some((tr0, tr1)) = r.intersect(ray) {
                (tr0.min(t0), tr1.max(t1))
            } else {
                (t0, t1)
            }
        });

        if t0 < t1 { Some((t0, t1)) } else { None }
    }
}

impl VolumeRegion for AggregateVolumeRegion {
    fn sigma_a(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
        self.regions.iter().fold(Spectrum::from(0.0), |old_s, r| {
            old_s + r.sigma_a(p, w, time)
        })
    }

    fn sigma_s(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
        self.regions.iter().fold(Spectrum::from(0.0), |old_s, r| {
            old_s + r.sigma_s(p, w, time)
        })
    }
        
    fn l_ve(&self, p: &Point, w: &Vector, time: f32) -> Spectrum {
        self.regions.iter().fold(Spectrum::from(0.0), |old_s, r| {
            old_s + r.l_ve(p, w, time)
        })
    }

    fn p(&self, p: &Point, w: &Vector, wp: &Vector, t: f32) -> f32 {
        // For some reason p is averaged here
        let (ph, sum) = self.regions.iter().fold((0.0, 0.0), |(old_p, old_sum), r| {
            let wt = r.sigma_s(p, w, t).y();
            (old_p + wt * r.p(p, w, wp, t), old_sum + wt)
        });
        ph / sum
    }

    fn tau(&self, ray: &Ray, t0: f32, t1: f32) -> Spectrum {
        self.regions.iter().fold(Spectrum::from(0.0), |old_s, r| {
            old_s + r.tau(ray, t0, t1)
        })
    }
}
