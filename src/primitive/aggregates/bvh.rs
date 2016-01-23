use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use geometry::point::Point;
use primitive::Primitive;
use primitive::FullyRefinable;

use utils::partition_by;

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

fn split_equal_counts(dim: usize, prims: Vec<BVHPrimitiveInfo>)
                      -> (Vec<BVHPrimitiveInfo>, Vec<BVHPrimitiveInfo>) {
    let mut mut_prims = prims;
    partition_by(&mut mut_prims, |p| p.centroid[dim]);

    let n = mut_prims.len();
    let (left, right) : (Vec<_>, Vec<_>) =
        mut_prims.into_iter().enumerate().partition(|&(i, _)| i < (n / 2));

    (left.into_iter().map(|(_, p)| p).collect(),
     right.into_iter().map(|(_, p)| p).collect())
}

const NUM_BUCKETS: usize = 12;
fn recursive_build(prims: Vec<BVHPrimitiveInfo>,
                   max_prims_in_node: usize,
                   sm: SplitMethod) -> (BVHNode, Vec<Primitive>) {
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
                    SplitMethod::EqualCounts => split_equal_counts(dim, prims),
                    SplitMethod::SAH => {
                        if prims.len() <= 4 {
                            split_equal_counts(dim, prims)
                        } else {

                            // Allocate BucketInfo for SAH partition buckets
                            let mut buckets = vec![(0, BBox::new()); NUM_BUCKETS];

                            let bucket_for_prim = |p: &BVHPrimitiveInfo| {
                                let b = {
                                    let pdist = p.centroid[dim] - centroid_bounds.p_min[dim];
                                    let dist = centroid_bounds.p_max[dim] - centroid_bounds.p_min[dim];
                                    ((NUM_BUCKETS as f32) * (pdist / dist)) as usize
                                };

                                if b == NUM_BUCKETS {
                                    NUM_BUCKETS - 1
                                } else {
                                    b
                                }
                            };

                            // Initialize BucketInfo for SAH partition buckets
                            for p in prims.iter() {
                                let b = bucket_for_prim(p);
                                buckets[b].0 += 1;
                                buckets[b].1.union_with(&p.bounds);
                            }

                            // Compute costs for splitting after each bucket
                            let cost = {
                                let mut costs = [0f32; NUM_BUCKETS - 1];
                                for i in 0..(NUM_BUCKETS - 1) {
                                    let (cnt0, b0) = buckets.iter().take(i+1)
                                        .fold((0, BBox::new()),
                                              |(fc, fb), &(ref c, ref b)| (fc + c, fb.unioned_with_ref(b)));
                                    let (cnt1, b1) = buckets.iter().skip(i+1)
                                        .fold((0, BBox::new()),
                                              |(fc, fb), &(ref c, ref b)| (fc + c, fb.unioned_with_ref(b)));

                                    costs[i] = (cnt0 as f32) * b0.surface_area();
                                    costs[i] += (cnt1 as f32) * b1.surface_area();
                                    costs[i] *= 0.125;
                                    costs[i] /= bbox.surface_area();
                                }
                                costs
                            };

                            // Find bucket to split at that minimizes SAH metric
                            let (min_cost_split, _) = cost.iter().enumerate()
                                .fold((0, ::std::f32::MAX), |(s, mc), (i, &c)| {
                                    if c < mc { (i, c) } else { (s, mc) }
                                });

                            // Either create leaf or split primitives at selected SAH bucket
                            if prims.len() > max_prims_in_node || min_cost_split < prims.len() {
                                prims.into_iter().partition(|p| { bucket_for_prim(p) < min_cost_split })
                            } else {
                                // Make leaf node
                                let node = BVHNode::Leaf {
                                    bounds: bbox,
                                    first_prim_offset: 0,
                                    num_primitives: num_prims
                                };

                                let new_prims = prims
                                    .into_iter()
                                    .map(|BVHPrimitiveInfo { primitive, .. }| primitive)
                                    .collect();

                                return (node, new_prims);
                            }
                        }
                    }
                }
            };

            let (left, mut ordered_left) = recursive_build(p1, max_prims_in_node, sm);
            let (mut right, mut ordered_right) = recursive_build(p2, max_prims_in_node, sm);

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

        let (tree, ordered_prims) = recursive_build(build_data, mp, split_method);

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
