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

#[derive(Clone, Debug)]
pub struct KDTreeAccelerator {
    isect_cost: i32,
    traversal_cost: i32,
    max_prims: usize,
    max_depth: usize,
    empty_bonus: f32,
    bounds: BBox,
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
        let prim_nums: Vec<usize> = (0..num_prims).collect();

        // Start recursive construction of kd-tree

        // Build kd-tree for accelerator
        KDTreeAccelerator {
            isect_cost: icost,
            traversal_cost: tcost,
            max_prims: maxp,
            max_depth: maxd,
            empty_bonus: ebonus,
            bounds: bounds,
            primitives: prims,
            nodes: Vec::new()
        }
    }
}
