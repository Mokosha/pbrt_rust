use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use geometry::point::Point;
use primitive::Primitive;
use primitive::FullyRefinable;

#[derive(Clone, Debug, PartialEq, Copy)]
enum SplitMethod {
    Middle,
    EqualCounts,
    SAH
}

#[derive(Clone, Debug)]
enum BVHNode {
    Leaf {
        bounds: BBox,
        first_prim_offset: usize,
        num_primitives: usize
    },
    Inner {
        bounds: BBox,
        child1: Box<BVHNode>,
        child2: Box<BVHNode>,
        split_axis: usize,
        num_nodes: usize
    }
}

impl BVHNode {
    fn num_nodes(&self) -> usize {
        match self {
            &BVHNode::Leaf { .. }=> 1,
            &BVHNode::Inner { num_nodes, .. } => num_nodes
        }
    }

    fn offset(&mut self, off: usize) {
        match self {
            &mut BVHNode::Leaf { first_prim_offset, .. } => {
                first_prim_offset + off;
            },

            &mut BVHNode::Inner { ref mut child1, ref mut child2, .. } => {
                child1.offset(off);
                child2.offset(off);
            }
        }
    }

    fn bounds<'a>(&'a self) -> &'a BBox {
        match self {
            &BVHNode::Leaf { ref bounds, .. } => bounds,
            &BVHNode::Inner { ref bounds, .. } => bounds
        }
    }
}

struct BVHPrimitiveInfo {
    primitive: Primitive,
    centroid: Point,
    bounds: BBox,
}

impl BVHPrimitiveInfo {
    fn new(p: Primitive, bnds: BBox) -> BVHPrimitiveInfo {
        BVHPrimitiveInfo {
            primitive: p,
            centroid: (&bnds.p_min + &bnds.p_max) * 0.5,
            bounds: bnds
        }
    }

    fn prim<'a>(&'a self) -> &'a Primitive { &self.primitive }
    fn centroid<'a>(&'a self) -> &'a Point { &self.centroid }
}

#[derive(Clone, Debug)]
pub struct BVHAccel {
    root: BVHNode,
    primitives: Vec<Primitive>,
}

fn split_middle(centroid_bounds: BBox, dim: usize, prims: Vec<BVHPrimitiveInfo>)
                -> (Vec<BVHPrimitiveInfo>, Vec<BVHPrimitiveInfo>) {
    let p_mid = 0.5 * (centroid_bounds.p_min[dim] + centroid_bounds.p_max[dim]);
    prims.into_iter().partition(|p| p.centroid()[dim] < p_mid)
}

fn split_equal_counts(prims: Vec<BVHPrimitiveInfo>) -> (Vec<BVHPrimitiveInfo>, Vec<BVHPrimitiveInfo>) {
    unimplemented!();
}

fn split_surface_area(prims: Vec<BVHPrimitiveInfo>) -> (Vec<BVHPrimitiveInfo>, Vec<BVHPrimitiveInfo>) {
    unimplemented!();
}

fn recursive_build(prims: Vec<BVHPrimitiveInfo>, sm: SplitMethod)
                   -> (BVHNode, Vec<Primitive>) {
    let bbox = prims.iter().fold(BBox::new(), |b, p| {
        b.unioned_with(p.prim().world_bound())
    });
    let num_prims = prims.len();

    if num_prims == 1 {
        // Make leaf node
        let node = BVHNode::Leaf {
            bounds: bbox,
            first_prim_offset: 0,
            num_primitives: num_prims
        };

        (node, prims.into_iter().map(|BVHPrimitiveInfo { primitive, .. }| primitive).collect())
    } else {
        let centroid_bounds = prims.iter().fold(BBox::new(), |b, p| {
            b.unioned_with_ref(p.centroid())
        });
        let dim = centroid_bounds.max_extent();

        if centroid_bounds.p_min[dim] == centroid_bounds.p_max[dim] {
            // Make leaf node
            let node = BVHNode::Leaf {
                bounds: bbox,
                first_prim_offset: 0,
                num_primitives: num_prims
            };

            (node, prims.into_iter().map(|BVHPrimitiveInfo { primitive, .. }| primitive).collect())
        } else {
            // Partition primitives based on split method
            let (p1, p2) = {
                match sm {
                    SplitMethod::Middle => split_middle(centroid_bounds, dim, prims),
                    SplitMethod::EqualCounts => split_equal_counts(prims),
                    SplitMethod::SAH => split_surface_area(prims)
                }
            };

            let (left, mut ordered_left) = recursive_build(p1, sm);
            let (mut right, mut ordered_right) = recursive_build(p2, sm);

            right.offset(ordered_left.len());
            let num_nodes = right.num_nodes() + left.num_nodes();
            let node = BVHNode::Inner {
                bounds: left.bounds().clone().unioned_with_ref(right.bounds()),
                child1: Box::new(left),
                child2: Box::new(right),
                split_axis: dim,
                num_nodes: num_nodes + 1
            };

            ordered_left.append(&mut ordered_right);
            (node, ordered_left)
        }
    }
}

impl BVHAccel {
    fn new(p: Vec<Primitive>, mp: usize, sm: &'static str) -> BVHAccel {
        let prims = p.into_iter().fold(Vec::new(), |mut ps, prim| {
            ps.append(&mut prim.fully_refine());
            ps
        });

        let split_method = match sm {
            "sah" => SplitMethod::SAH,
            "middle" => SplitMethod::Middle,
            "equal" => SplitMethod::EqualCounts,
            _ => {
                println!("Warning: BVH split method {} unknown. Using \"SAH\"", sm);
                SplitMethod::SAH
            }
        };

        let build_data: Vec<_> = prims.into_iter().enumerate().map(|(i, p)| {
            let bbox = p.world_bound();
            BVHPrimitiveInfo::new(p, bbox)
        }).collect();

        let (tree, ordered_prims) = recursive_build(build_data, split_method);

        BVHAccel {
            root: tree,
            primitives: ordered_prims,
        }
    }
}

#[cfg(test)]
mod tests  {
    use super::*;

    #[test]
    #[ignore]
    fn it_can_be_created() {
        unimplemented!();
    }
}
