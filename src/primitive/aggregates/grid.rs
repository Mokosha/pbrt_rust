use std::sync::{Arc, Weak};

use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use geometry::point::Point;
use geometry::vector::Vector;
use primitive::Primitive;
use primitive::FullyRefinable;

use utils::Clamp;

#[derive(Debug, Clone)]
struct Voxel {
    primitives: Vec<Weak<Primitive>>,
    all_can_intersect: bool
}

impl Voxel {
    fn new(p: Weak<Primitive>) -> Voxel {
        Voxel {
            primitives: vec![p],
            all_can_intersect: false
        }
    }

    fn add_primitive(&mut self, p: Weak<Primitive>) {
        self.primitives.push(p)
    }
}

#[derive(Debug, Clone)]
pub struct GridAccelerator {
    primitives: Vec<Arc<Primitive>>,
    num_voxels: [usize; 3],
    bounds: BBox,
    width: Vector,
    inv_width: Vector,
    voxels: Vec<Option<Voxel>>
}

impl GridAccelerator {
    pub fn new(p: Vec<Primitive>, refine_immediately: bool) -> GridAccelerator {
        // Initialize primitives with primitives for grid
        let prims = {
            if refine_immediately {
                p.into_iter().fold(Vec::new(), |mut ps, prim| {
                    ps.append(&mut prim.fully_refine().into_iter().map(Arc::new).collect());
                    ps
                })
            } else {
                p.into_iter().map(Arc::new).collect()
            }
        };

        // Compute bounds and choose grid resolution
        let bounds = prims.iter().fold(BBox::new(), |b, prim| {
            b.unioned_with(prim.world_bound())
        });
        let delta = &bounds.p_max - &bounds.p_min;

        // Find voxels_per_unit_dist for grid
        let voxels_per_unit_dist = {
            let max_axis = bounds.max_extent();
            let inv_max_width = 1f32 / delta[max_axis];
            let cube_root = 3f32 * (prims.len() as f32).powf(1f32 / 3f32);
            cube_root * inv_max_width
        };

        let num_voxels = [
            ((delta[0] * voxels_per_unit_dist) as usize).clamp(1, 64),
            ((delta[1] * voxels_per_unit_dist) as usize).clamp(1, 64),
            ((delta[2] * voxels_per_unit_dist) as usize).clamp(1, 64)];

        // Compute voxel widths and allocate voxels
        let mut voxel_width = Vector::new();
        let mut inv_voxel_width = Vector::new();
        for axis in 0..3 {
            voxel_width[axis] = delta[axis] / (num_voxels[axis] as f32);
            inv_voxel_width[axis] =
                if voxel_width[axis] == 0.0 {
                    0.0
                } else {
                    1f32 / voxel_width[axis]
                };
        }

        let mut grid = GridAccelerator {
            primitives: prims,
            num_voxels: num_voxels,
            bounds: bounds,
            width: voxel_width,
            inv_width: inv_voxel_width,
            voxels: {
                (0..(num_voxels[0] * num_voxels[1] * num_voxels[2]))
                    .map(|_| None)
                    .collect()
            }
        };

        // Add primitives to grid voxels
        for prim in grid.primitives.iter() {
            // Find voxel extent of primitive
            let pb = prim.world_bound();

            let vmin = grid.point_to_voxel(&pb.p_min);
            let vmax = grid.point_to_voxel(&(&pb.p_max - Vector::new_with(1e-6, 1e-6, 1e-6)));

            // Add primitive to overlapping voxels
            for z in vmin[2]..(vmax[2] + 1) {
                for y in vmin[1]..(vmax[1] + 1) {
                    for x in vmin[0]..(vmax[0] + 1) {
                        let o = grid.offset(x, y, z);
                        if grid.voxels[o].is_some() {
                            grid.voxels[o].as_mut().unwrap().add_primitive(Arc::downgrade(prim));
                        } else {
                            grid.voxels[o] = Some(Voxel::new(Arc::downgrade(prim)));
                        }
                    }
                }
            }
        }

        // Create reader-writer mutex for grid

        grid
    }

    fn offset(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.num_voxels[0] * self.num_voxels[1] + y * self.num_voxels[0] + x
    }

    fn point_to_voxel(&self, p: &Point) -> [usize; 3] {
        [self.pos_to_voxel(p, 0),
         self.pos_to_voxel(p, 1),
         self.pos_to_voxel(p, 2)]
    }

    fn voxel_to_point(&self, p: [usize; 3]) -> Point {
        Point::new_with(
            self.voxel_to_pos(p[0], 0),
            self.voxel_to_pos(p[1], 1),
            self.voxel_to_pos(p[2], 2))
    }

    fn pos_to_voxel(&self, p: &Point, axis: usize) -> usize {
        (((p[axis] - self.bounds.p_min[axis]) * self.inv_width[axis]) as usize)
            .clamp(0, self.num_voxels[axis] - 1)
    }

    fn voxel_to_pos(&self, p: usize, axis: usize) -> f32 {
        self.bounds.p_min[axis] + (p as f32) * self.width[axis]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bbox::BBox;
    use primitive::Primitive;
    use geometry::point::Point;
    use geometry::vector::Vector;
    use intersection::Intersectable;
    use ray::Ray;
    use shape::Shape;
    use transform::transform::Transform;

    fn get_spheres() -> Vec<Primitive> { vec![
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 0.0, 0.0)),
            Transform::translate(&Vector::new_with(0.0, 0.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 0.0, 0.0)),
            Transform::translate(&Vector::new_with(-2.0, 0.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 2.0, 0.0)),
            Transform::translate(&Vector::new_with(0.0, -2.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 2.0, 0.0)),
            Transform::translate(&Vector::new_with(-2.0, -2.0, 0.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 0.0, 2.0)),
            Transform::translate(&Vector::new_with(0.0, 0.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 0.0, 2.0)),
            Transform::translate(&Vector::new_with(-2.0, 0.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(0.0, 2.0, 2.0)),
            Transform::translate(&Vector::new_with(0.0, -2.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0)),
        Primitive::geometric(Shape::sphere(
            Transform::translate(&Vector::new_with(2.0, 2.0, 2.0)),
            Transform::translate(&Vector::new_with(-2.0, -2.0, -2.0)),
            false, 1.0, -1.0, 1.0, 360.0))]
    }
            
    #[test]
    fn it_can_be_created() {
        let g = GridAccelerator::new(get_spheres(), false);

        assert_eq!(g.primitives.len(), 8);

        // The number of voxels is six instead of two because the heuristic for
        // choosing the number of voxels is chosen based on 3 * sqrt(N) where
        // N is the number of primitives...
        assert_eq!(g.num_voxels[0], 6);
        assert_eq!(g.num_voxels[1], 6);
        assert_eq!(g.num_voxels[2], 6);

        assert_eq!(g.bounds, BBox::new_with(Point::new_with(-1.0, -1.0, -1.0),
                                            Point::new_with(3.0, 3.0, 3.0)));

        let sz = 4.0 / 6.0;
        let inv_sz = 6.0 / 4.0;
        assert_eq!(g.width, Vector::new_with(sz, sz, sz));
        assert_eq!(g.inv_width, Vector::new_with(inv_sz, inv_sz, inv_sz));

        assert_eq!(g.voxels.len(), 6*6*6);
    }

    #[test]
    fn it_can_place_primitives() {
        let g = GridAccelerator::new(get_spheres(), false);

        // Each voxel should have exactly one sphere in it...
        for z in 0..6 { for y in 0..6 { for x in 0..6 {
            let o = g.offset(x, y, z);
            assert_eq!(g.voxels[o].as_ref().unwrap().primitives.len(), 1);

            // Furthermore, the primitive should be the corresponding
            // sphere...
            let px = if x < 3 { 0.0 } else { 2.0 };
            let py = if y < 3 { 0.0 } else { 2.0 };
            let pz = if z < 3 { 0.0 } else { 2.0 };

            let prim = g.voxels[o].as_ref().unwrap().primitives[0].upgrade().unwrap();
            assert!(prim.intersect_p(&Ray::new_with(
                Point::new_with(px, py, pz), Vector::new_with(px - 1.0, py - 1.0, pz - 1.0), 0.0)));
        } } }
    }
}
