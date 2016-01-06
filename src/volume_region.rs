use bbox;

#[derive(Debug, Copy, Clone)]
pub struct VolumeRegion;

impl bbox::HasBounds for VolumeRegion {
    fn world_bound(&self) -> bbox::BBox { bbox::BBox::new() }
}
