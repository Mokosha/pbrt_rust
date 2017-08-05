use bbox::BBox;
use bbox::HasBounds;
use geometry::normal::Normalize;
use geometry::point::Point;
use geometry::vector::Dot;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Clamp;
use utils::Lerp;
use volume::density::internal::DensityRegion;
use volume::VolumeRegion;

#[derive(Clone, Debug, PartialEq)]
struct ExponentialDensity {
    sigma_a: Spectrum,
    sigma_s: Spectrum,
    g: f32,
    l_ve: Spectrum,
    world_to_volume: Transform,
    extent: BBox,
    a: f32,
    b: f32,
    up_dir: Vector,
}

impl ExponentialDensity {
    pub fn new(sa: Spectrum, ss: Spectrum, g: f32, emit: Spectrum, e: BBox,
               v2w: Transform, aa: f32, bb: f32, up: Vector) -> ExponentialDensity {
        ExponentialDensity {
            sigma_a: sa,
            sigma_s: ss,
            g: g,
            l_ve: emit,
            world_to_volume: v2w.invert(),
            extent: e,
            a: aa, b: bb,
            up_dir: up.normalize()
        }
    }
}

impl HasBounds for ExponentialDensity {
    fn world_bound(&self) -> BBox {
        self.world_to_volume.inverse().t(&self.extent)
    }
}

impl Intersectable<(f32, f32)> for ExponentialDensity {
    fn intersect(&self, r: &Ray) -> Option<(f32, f32)> {
        self.extent.intersect(&self.world_to_volume.t(r))
    }
}

impl DensityRegion for ExponentialDensity {
    fn get_sig_a(&self) -> Spectrum { self.sigma_a.clone() }
    fn get_sig_s(&self) -> Spectrum { self.sigma_s.clone() }
    fn get_le(&self) -> Spectrum { self.l_ve.clone() }
    fn get_g(&self) -> f32 { self.g }
    fn world_to_volume(&self) -> Transform { self.world_to_volume.clone() }

    fn density(&self, p_obj: Point) -> f32 {
        if !self.extent.inside(&p_obj) {
            return 0.0;
        }

        let height = (p_obj - self.extent.p_min.clone()).dot(&self.up_dir);
        self.a * (-self.b * height).exp()
    }
}
