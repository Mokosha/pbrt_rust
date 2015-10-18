use bbox;

#[derive(Debug, Copy, Clone)]
pub struct VolumeRegion;

impl bbox::HasBounds for VolumeRegion {
    fn get_bounds(&self) -> bbox::BBox { bbox::BBox::new() }
}
