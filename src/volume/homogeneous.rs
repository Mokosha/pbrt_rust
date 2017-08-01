use bbox::BBox;
use bbox::HasBounds;
use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use volume::VolumeRegion;

use volume::phase_schlick;

#[derive(Clone, Debug, PartialEq)]
pub struct HomogeneousVolumeDensity {
    sig_a: Spectrum,
    sig_s: Spectrum,
    g: f32,
    le: Spectrum,
    extent: BBox,
    world_to_volume: Transform,
}

impl HomogeneousVolumeDensity {
    pub fn new(sig_a: Spectrum, sig_s: Spectrum, g: f32, le: Spectrum, extent: BBox,
               world_to_volume: Transform) -> HomogeneousVolumeDensity {
        HomogeneousVolumeDensity { sig_a, sig_s, g, le, extent, world_to_volume }
    }
}

impl HasBounds for HomogeneousVolumeDensity {
    fn world_bound(&self) -> BBox { self.world_to_volume.inverse().t(&self.extent) }
}

impl Intersectable<(f32, f32)> for HomogeneousVolumeDensity {
    fn intersect(&self, r: &Ray) -> Option<(f32, f32)> {
        self.extent.intersect(&self.world_to_volume.t(r))
    }
}

impl VolumeRegion for HomogeneousVolumeDensity {
    fn sigma_a(&self, p: &Point, _: &Vector, _: f32) -> Spectrum {
        if self.extent.inside(&self.world_to_volume.t(p)) {
            self.sig_a.clone()
        } else {
            Spectrum::from(0.0)
        }
    }

    fn sigma_s(&self, p: &Point, _: &Vector, _: f32) -> Spectrum {
        if self.extent.inside(&self.world_to_volume.t(p)) {
            self.sig_s.clone()
        } else {
            Spectrum::from(0.0)
        }
    }
        
    fn l_ve(&self, p: &Point, _: &Vector, _: f32) -> Spectrum {
        if self.extent.inside(&self.world_to_volume.t(p)) {
            self.le.clone()
        } else {
            Spectrum::from(0.0)
        }
    }

    fn p(&self, p: &Point, w: &Vector, wp: &Vector, _: f32) -> f32 {
        if self.extent.inside(&self.world_to_volume.t(p)) {
            phase_schlick(w, wp, self.g)
        } else {
            0.0
        }
    }

    fn tau(&self, r: &Ray, _: f32, _: f32) -> Spectrum {
        if let Some((t0, t1)) = self.intersect(r) {
            r.point_at(t0).distance(&r.point_at(t1)) *
                (self.sig_s.clone() + self.sig_a.clone())
        } else {
            Spectrum::from(0.0)
        }
    }
}
