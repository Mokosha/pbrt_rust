use bbox::BBox;
use bbox::HasBounds;
use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use ray::Ray;
use spectrum::Spectrum;
use transform::transform::ApplyTransform;
use transform::transform::Transform;
use utils::Lerp;
use volume::density::internal::DensityRegion;
use volume::VolumeRegion;

#[derive(Clone, Debug, PartialEq)]
struct VolumeGridDensity {
    sigma_a: Spectrum,
    sigma_s: Spectrum,
    g: f32,
    l_ve: Spectrum,
    world_to_volume: Transform,
    nx: usize,
    ny: usize,
    nz: usize,
    extent: BBox,
    density: Vec<f32>,
}

impl VolumeGridDensity {
    pub fn new(sa: Spectrum, ss: Spectrum, g: f32, emit: Spectrum, e: BBox,
               v2w: Transform, x: usize, y: usize, z: usize,
               d: Vec<f32>) -> VolumeGridDensity {
        assert_eq!(x * y * z, d.len());
        VolumeGridDensity {
            sigma_a: sa,
            sigma_s: ss,
            g: g,
            l_ve: emit,
            world_to_volume: v2w.invert(),
            nx: x, ny: y, nz: z,
            extent: e,
            density: d
        }
    }

    fn d(&self, _x: i32, _y: i32, _z: i32) -> f32 {
        let x = _x.clamp(0, (self.nx - 1) as i32) as usize;
        let y = _y.clamp(0, (self.ny - 1) as i32) as usize;
        let z = _z.clamp(0, (self.nz - 1) as i32) as usize;
        self.density[z * self.nx * self.ny + y * self.nx + x]
    }
}

impl HasBounds for VolumeGridDensity {
    fn world_bound(&self) -> BBox {
        self.world_to_volume.inverse().t(&self.extent)
    }
}

impl Intersectable<(f32, f32)> for VolumeGridDensity {
    fn intersect(&self, r: &Ray) -> Option<(f32, f32)> {
        self.extent.intersect(&self.world_to_volume.t(r))
    }
}

impl DensityRegion for VolumeGridDensity {
    fn get_sig_a(&self) -> Spectrum { self.sigma_a.clone() }
    fn get_sig_s(&self) -> Spectrum { self.sigma_s.clone() }
    fn get_le(&self) -> Spectrum { self.l_ve.clone() }
    fn get_g(&self) -> f32 { self.g }
    fn world_to_volume(&self) -> Transform { self.world_to_volume.clone() }

    fn density(&self, p_obj: Point) -> f32 {
        if !self.extent.inside(&p_obj) {
            return 0.0;
        }

        // Compute voxel coordinates and offsets for p_obj
        let mut vox = {
            let mut v = self.extent.offset(&p_obj);
            v.x = v.x * (self.nx as f32) - 0.5;
            v.y = v.y * (self.ny as f32) - 0.5;
            v.z = v.z * (self.nz as f32) - 0.5;
            v
        };

        let vx = vox.x.floor() as i32;
        let vy = vox.y.floor() as i32;
        let vz = vox.z.floor() as i32;

        let dx = vox.x - (vx as f32);
        let dy = vox.y - (vy as f32);
        let dz = vox.z - (vz as f32);

        // Trilinearly interpolate density values to compute local density
        let c0 = self.d(vx    , vy    , vz    );
        let c1 = self.d(vx + 1, vy    , vz    );
        let c2 = self.d(vx    , vy + 1, vz    );
        let c3 = self.d(vx + 1, vy + 1, vz    );
        let c4 = self.d(vx    , vy    , vz + 1);
        let c5 = self.d(vx + 1, vy    , vz + 1);
        let c6 = self.d(vx    , vy + 1, vz + 1);
        let c7 = self.d(vx + 1, vy + 1, vz + 1);

        let d00 = c0.lerp_with(c1, dx);
        let d10 = c2.lerp_with(c3, dx);
        let d01 = c4.lerp_with(c5, dx);
        let d11 = c6.lerp_with(c7, dx);

        let d0 = d00.lerp_with(d10, dy);
        let d1 = d01.lerp_with(d11, dy);

        d0.lerp_with(d1, dz)
    }
}
