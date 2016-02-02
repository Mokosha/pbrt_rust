use bbox::BBox;
use bbox::HasBounds;
use bbox::Union;
use primitive::Primitive;

#[derive(Clone, Copy, PartialEq, Debug)]
enum SplitAxis { X, Y, Z }

// !SPEED! Rust uses tagged unions to construct enum values. That
// means that each of these values will be 8 bytes larger than they
// need to be to hold the tag, reaching 16 bytes instead of eight.
// We can get tricky with data layout to make the smaller to improve
// cache performance. (See section 4.5.1)
#[derive(Clone, PartialEq, Debug)]
enum KDAccelNode {
    InteriorX {
        split: f32,
        above_child: usize
    },
    InteriorY {
        split: f32,
        above_child: usize
    },
    InteriorZ {
        split: f32,
        above_child: usize
    },
    Leaf(Vec<usize>)
}

impl KDAccelNode {
    fn leaf(prim_nums: Vec<usize>) -> KDAccelNode {
        KDAccelNode::Leaf(prim_nums)
    }

    fn interior(axis: SplitAxis, ac: usize, s: f32) -> KDAccelNode {
        match axis {
            SplitAxis::X => KDAccelNode::InteriorX { split: s, above_child: ac },
            SplitAxis::Y => KDAccelNode::InteriorY { split: s, above_child: ac },
            SplitAxis::Z => KDAccelNode::InteriorZ { split: s, above_child: ac },
        }
    }

    fn offset(&mut self, off: usize) {
        match self {
            &mut KDAccelNode::InteriorX { ref mut above_child, .. } => *above_child += off,
            &mut KDAccelNode::InteriorY { ref mut above_child, .. } => *above_child += off,
            &mut KDAccelNode::InteriorZ { ref mut above_child, .. } => *above_child += off,
            _ => { } // Leaf nodes needn't do anything
        }
    }

    fn split_pos(&self) -> f32 {
        match self {
            &KDAccelNode::InteriorX { split, .. } => split,
            &KDAccelNode::InteriorY { split, .. } => split,
            &KDAccelNode::InteriorZ { split, .. } => split,
            _ => panic!("Leaf nodes have no split node...")
        }
    }

    fn split_axis(&self) -> SplitAxis {
        match self {
            &KDAccelNode::InteriorX {..} => SplitAxis::X,
            &KDAccelNode::InteriorY {..} => SplitAxis::Y,
            &KDAccelNode::InteriorZ {..} => SplitAxis::Z,
            _ => panic!("Leaf nodes have no split axis...")
        }
    }

    fn above_child(&self) -> usize {
        match self {
            &KDAccelNode::InteriorX { above_child, .. } => above_child,
            &KDAccelNode::InteriorY { above_child, .. } => above_child,
            &KDAccelNode::InteriorZ { above_child, .. } => above_child,
            _ => panic!("Leaf nodes have no above_child...")
        }
    }

    fn is_leaf(&self) -> bool {
        match self {
            &KDAccelNode::Leaf(_) => true,
            _ => false
        }
    }

    fn num_prims(&self) -> usize {
        match self {
            &KDAccelNode::Leaf(ref prim_ids) => prim_ids.len(),
            _ => panic!("Interior nodes don't know how many primitives they have...")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum BoundEdge {
    Unknown,
    Start(f32, usize),
    End(f32, usize)
}

impl BoundEdge {
    fn is_start(&self) -> bool {
        match self {
            &BoundEdge::Start(_, _) => true,
            _ => false,
        }
    }

    fn is_end(&self) -> bool {
        match self {
            &BoundEdge::End(_, _) => true,
            _ => false,
        }
    }

    fn get_values(&self) -> (f32, usize) {
        match self {
            &BoundEdge::Unknown => panic!("Unknown bound edge!"),
            &BoundEdge::Start(x, y) => (x, y),
            &BoundEdge::End(x, y) => (x, y)
        }
    }
}

impl ::std::cmp::PartialOrd for BoundEdge {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        let (va, a_start): (f32, usize) = match self {
            &BoundEdge::Unknown => panic!("Unknown bound edge!"),
            &BoundEdge::Start(x, _) => (x, 0),
            &BoundEdge::End(x, _) => (x, 1)
        };

        let (vb, b_start): (f32, usize) = match other {
            &BoundEdge::Unknown => panic!("Unknown bound edge!"),
            &BoundEdge::Start(x, _) => (x, 0),
            &BoundEdge::End(x, _) => (x, 1)
        };

        if va == vb {
            a_start.partial_cmp(&b_start)
        } else {
            va.partial_cmp(&vb)
        }
    }
}

fn build_tree(icost: i32, tcost: i32, maxp: usize, ebonus: f32,
              bounds: BBox, prim_bounds: &Vec<BBox>, prim_nums: &mut [usize],
              depth: usize, edges: &mut [Vec<BoundEdge>], prims_scratch: &mut [usize],
              bad_refines: usize) -> Vec<KDAccelNode> {
    // Initialize leaf node if termination criteria are met
    let num_prims = prim_nums.len();
    if num_prims < maxp || depth == 0 {
        return vec![ KDAccelNode::leaf(prim_nums.iter().map(|x| *x).collect()) ];
    }

    // Choose split axis for interior node
    let mut best_offset = None;
    let mut best_cost = ::std::f32::MAX;
    let old_cost = (icost as f32) * (num_prims as f32);
    let total_SA = bounds.surface_area();
    let inv_total_SA = 1.0 / total_SA;
    let d = &bounds.p_max - &bounds.p_min;

    // Choose which axis to split along
    let mut axis = bounds.max_extent();
    let mut num_bad_refines = 0;

    for retries in 0..3 {
        let (n0, n1, tsplit, num_bad_refines) = {
            // Initialize edges for axis
            for (i, pn) in prim_nums.iter().map(|x| *x).enumerate() {
                let bbox = &prim_bounds[pn];
                edges[axis][2*i] = BoundEdge::Start(bbox.p_min[axis], pn);
                edges[axis][2*i + 1] = BoundEdge::End(bbox.p_max[axis], pn);
            }
            let (our_edges, _) = edges[axis].split_at_mut(2 * num_prims);
            our_edges.sort_by(|a, b| { a.partial_cmp(b).unwrap() });

            // Compute cost of all splits for axis to find best.
            let (mut num_below, mut num_above) = (0, num_prims);
            for (i, edge) in our_edges.iter().enumerate() {
                if edge.is_end() { num_above -= 1; }
                let (edge_t, _) = edge.get_values();
                if edge_t > bounds.p_min[axis] && edge_t < bounds.p_max[axis] {
                    // Compuite cost for split at ith edge
                    let other_axis_0 = (axis + 1) % 3;
                    let other_axis_1 = (axis + 2) % 3;

                    let compute_SA = |x| {
                        2.0 * (d[other_axis_0] * d[other_axis_1] +
                               x * (d[other_axis_0] * d[other_axis_1]))
                    };

                    let below_SA = compute_SA(edge_t - bounds.p_min[axis]);
                    let above_SA = compute_SA(bounds.p_max[axis] - edge_t);

                    let p_below = below_SA * inv_total_SA;
                    let p_above = above_SA * inv_total_SA;
                    let eb = if num_above == 0 || num_below == 0 { ebonus } else { 0.0 };
                    let cost =
                        (tcost as f32) +
                        (icost as f32) *
                        (1.0 - eb) *
                        (p_below * (num_below as f32) + p_above * (num_above as f32));

                    if cost < best_cost {
                        best_cost = cost;
                        best_offset = Some(i);
                    }
                }
                if edge.is_start() { num_below += 1; }
            }

            // Create leaf if no good splits were found
            if best_offset == None {
                axis = (axis + 1) % 3;
                continue;
            }

            if best_cost > old_cost {
                num_bad_refines += 1;
            }

            let prohibitively_costly = best_cost >= (4.0 * old_cost);
            let badly_refined_limit = (num_bad_refines + bad_refines) >= 3;
            if (prohibitively_costly && num_prims < 16) || badly_refined_limit {
                break;
            }

            // Classify primitives with respect to split
            let (mut n0, mut n1) = (0, 0);
            for i in 0..best_offset.unwrap() {
                if let BoundEdge::Start(_, pn) = our_edges[i] {
                    prim_nums[n0] = pn;
                    n0 += 1;
                }
            }
            for i in (best_offset.unwrap()+1)..(2*num_prims) {
                if let BoundEdge::End(_, pn) = our_edges[i] {
                    prims_scratch[n1] = pn;
                    n1 += 1;
                }
            }
            let (tsplit, _) = our_edges[best_offset.unwrap()].get_values();
            (n0, n1, tsplit, num_bad_refines)
        };

        let (prims0, _) = prim_nums.split_at_mut(n0);
        let (prims1, scratch) = prims_scratch.split_at_mut(n1);

        // Recursively initialize children nodes
        let mut bounds0 = bounds.clone();
        let mut bounds1 = bounds.clone();
        bounds0.p_max[axis] = tsplit;
        bounds1.p_min[axis] = tsplit;

        let mut left_children = build_tree(icost, tcost, maxp, ebonus, bounds0,
                                           prim_bounds, prims0, depth - 1,
                                           edges, scratch, bad_refines + num_bad_refines);

        let mut right_children = build_tree(icost, tcost, maxp, ebonus, bounds1,
                                            prim_bounds, prims1, depth - 1,
                                            edges, scratch, bad_refines + num_bad_refines);

        for c in left_children.iter_mut() {
            c.offset(1);
        }

        for c in right_children.iter_mut() {
            c.offset(left_children.len());
        }

        let split_axis = match axis {
            0 => SplitAxis::X,
            1 => SplitAxis::Y,
            2 => SplitAxis::Z,
            _ => panic!("Axis num out of range!")
        };

        let mut result = vec![ KDAccelNode::interior(split_axis, 1, tsplit) ];
        result.append(&mut left_children);
        result.append(&mut right_children);
        return result
    }

    vec![ KDAccelNode::leaf(prim_nums.iter().map(|x| *x).collect()) ]
}

#[derive(Clone, Debug)]
pub struct KDTreeAccelerator {
    primitives: Vec<Primitive>,
    nodes: Vec<KDAccelNode>
}

impl KDTreeAccelerator {
    fn new(prims: Vec<Primitive>, icost: i32, tcost: i32, ebonus: f32,
           maxp: usize, maxd: usize) -> KDTreeAccelerator {
        let num_prims = prims.len();
        let max_depth = if maxd == 0 {
            // Just some randomly chosen numbers apparently? (p.232)
            ((num_prims as f32).log2() * 1.3 + 8.0).round() as usize
        } else {
            maxd
        };

        // Compute bounds for kd-tree construction
        let mut prim_bounds = Vec::with_capacity(num_prims);
        let bounds = prims.iter().fold(BBox::new(), |b, p| {
            let pb = p.world_bound();
            prim_bounds.push(pb.clone());
            b.unioned_with(pb)
        });

        // Initialize prim_nums for kd-tree construction
        let mut prim_nums: Vec<usize> = (0..num_prims).collect();

        // Allocate data
        let mut edges = vec![
            vec![ BoundEdge::Unknown; 2 * num_prims ],
            vec![ BoundEdge::Unknown; 2 * num_prims ],
            vec![ BoundEdge::Unknown; 2 * num_prims ]];
        let mut prims_scratch = vec![ 0, (max_depth + 1) * num_prims ];

        // Start recursive construction of kd-tree
        let nodes = build_tree(icost, tcost, maxp, ebonus, bounds.clone(),
                               &prim_bounds, &mut prim_nums, max_depth,
                               &mut edges, &mut prims_scratch, 0);

        // Build kd-tree for accelerator
        KDTreeAccelerator {
            primitives: prims,
            nodes: nodes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_be_created() {
        unimplemented!()
    }
}
