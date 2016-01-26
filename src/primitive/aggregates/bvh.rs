use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use geometry::point::Point;
use geometry::vector::Vector;
use intersection::Intersectable;
use intersection::Intersection;
use primitive::Primitive;
use primitive::FullyRefinable;
use ray::Ray;

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
            &mut BVHNode::Leaf { ref mut first_prim_offset, .. } => {
                *first_prim_offset += off;
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

fn split_surface_area_heuristic(total_bounds: BBox, centroid_bounds: BBox, mp: usize,
                                dim: usize, prims: Vec<BVHPrimitiveInfo>)
                                -> Result<(Vec<BVHPrimitiveInfo>, Vec<BVHPrimitiveInfo>),
                                          (BVHNode, Vec<Primitive>)>
{
    let num_prims = prims.len();
    if num_prims <= 4 {
        return Ok(split_equal_counts(dim, prims));
    }

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

            let b0sa = (cnt0 as f32) * b0.surface_area();
            let b1sa = (cnt1 as f32) * b1.surface_area();
            let sc = 0.125;
            let tsa = total_bounds.surface_area();

            costs[i] = sc * (b0sa + b1sa) / tsa;
        }
        costs
    };

    // Find bucket to split at that minimizes SAH metric
    let (min_cost_split, min_cost) = cost
        .iter()
        .enumerate()
        .fold((0, ::std::f32::MAX), |(s, mc), (i, &c)| {
            if c < mc { (i, c) } else { (s, mc) }
        });

    // Either create leaf or split primitives at selected SAH bucket
    if mp < num_prims || (min_cost as usize) < num_prims {
        Ok(prims.into_iter().partition(|p| { bucket_for_prim(p) <= min_cost_split }))
    } else {
        // Make leaf node
        let node = BVHNode::Leaf {
            bounds: total_bounds,
            first_prim_offset: 0,
            num_primitives: num_prims
        };

        let new_prims = prims
            .into_iter()
            .map(|BVHPrimitiveInfo { primitive, .. }| primitive)
            .collect();

        Err((node, new_prims))
    }
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
                        match split_surface_area_heuristic(bbox, centroid_bounds,
                                                           max_prims_in_node, dim, prims) {
                            Ok(split) => split,
                            Err(node_result) => return node_result
                        }
                    }
                }
            };

            assert!(p1.len() > 0);
            assert!(p2.len() > 0);

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

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub enum PackedBVHNode {
    Leaf {
        bounds: BBox,
        prim_offset: usize,
        num_prims: usize
    },
    Inner {
        bounds: BBox,
        second_child_offset: usize,
        axis: usize
    }
}

#[cfg(not(test))]
#[derive(Clone, Debug, PartialEq)]
enum PackedBVHNode {
    Leaf {
        bounds: BBox,
        prim_offset: usize,
        num_prims: usize
    },
    Inner {
        bounds: BBox,
        second_child_offset: usize,
        axis: usize
    }
}

impl PackedBVHNode {
    fn bounds<'a>(&'a self) -> &'a BBox {
        match self {
            &PackedBVHNode::Leaf { ref bounds, .. } => bounds,
            &PackedBVHNode::Inner { ref bounds, .. } => bounds
        }
    }

    fn flattenBVHTree(node: &BVHNode, nodes: &mut Vec<PackedBVHNode>, offset: usize) -> usize {
        match node {
            &BVHNode::Leaf { ref bounds, first_prim_offset, num_primitives } => {
                let n = PackedBVHNode::Leaf {
                    bounds: bounds.clone(),
                    prim_offset: first_prim_offset,
                    num_prims: num_primitives
                };

                nodes.push(n);
                offset + 1
            },

            &BVHNode::Inner { ref bounds, ref child1, ref child2, split_axis, num_nodes } => {
                let empty_interior = PackedBVHNode::Inner {
                    bounds: bounds.clone(),
                    second_child_offset: 0,
                    axis: split_axis
                };
                nodes.push(empty_interior);

                let c1_offset = PackedBVHNode::flattenBVHTree(child1, nodes, offset + 1);
                let c2_offset = PackedBVHNode::flattenBVHTree(child2, nodes, c1_offset);

                if let &mut PackedBVHNode::Inner {
                    ref mut second_child_offset, .. } = &mut nodes[offset] {

                    assert_eq!(*second_child_offset, 0);

                    *second_child_offset = c1_offset;
                }

                c2_offset
            }
        }
    }

    fn linearize(root: BVHNode) -> Vec<PackedBVHNode> {
        let mut nodes = Vec::with_capacity(root.num_nodes());
        let result = PackedBVHNode::flattenBVHTree(&root, &mut nodes, 0);
        assert_eq!(result, nodes.len());
        assert_eq!(result, root.num_nodes());
        nodes
    }
}

#[derive(Clone, Debug)]
pub struct BVHAccel {
    nodes: Vec<PackedBVHNode>,
    primitives: Vec<Primitive>,
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
            nodes: PackedBVHNode::linearize(tree),
            primitives: ordered_prims,
        }
    }
}

impl Intersectable for BVHAccel {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if self.nodes.len() == 0 { return None; }

        // !SPEED! used in speedup for ray-bbox intersection. See p. 225
        // let origin = ray.point_at(ray.min_t);
        let inv_dir = Vector::new_with(1f32 / ray.d.x, 1f32 / ray.d.y, 1f32 / ray.d.z);
        let dir_is_neg = [ inv_dir.x < 0.0, inv_dir.y < 0.0, inv_dir.z < 0.0 ];

        // Follow ray through BVH nodes to find primitive intersections
        let mut todo = Vec::with_capacity(64);
        todo.push(0);

        let mut isect = None;
        while let Some(node_num) = todo.pop() {

            // !SPEED! This can be accelerated, see p.225
            if !self.nodes[node_num].bounds().intersect_p(ray) {
                continue;
            }

            match &self.nodes[node_num] {
                &PackedBVHNode::Leaf { prim_offset, num_prims, ..} => {
                    // Intersect ray wiuth primitives in leaf BVH node
                    for i in 0..num_prims {
                        isect = self.primitives[prim_offset + i].intersect(ray);
                    }
                },
                &PackedBVHNode::Inner { second_child_offset, axis, .. } => {
                    // Put far BVH node on todo stack, advance to near node
                    if dir_is_neg[axis] {
                        todo.push(node_num + 1);
                        todo.push(second_child_offset);
                    } else {
                        todo.push(second_child_offset);
                        todo.push(node_num + 1);
                    }
                }
            }
        }

        isect
    }
}

#[cfg(test)]
mod tests  {
    use super::*;
    use primitive::aggregates::tests::get_spheres;

    #[test]
    fn it_can_be_created() {
        for sm in ["sah", "middle", "equal"].iter() {
            let bvh = BVHAccel::new(get_spheres(), 1, sm);
            assert_eq!(bvh.primitives.len(), 8);

            let mut prims = Vec::with_capacity(8);
            for n in bvh.nodes.iter() {
                if let &PackedBVHNode::Leaf { prim_offset, num_prims, .. } = n {
                    prims.push(prim_offset);
                    assert_eq!(num_prims, 1);
                }
            }

            prims.sort();
            assert_eq!(prims, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        }
    }
}
