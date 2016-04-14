use geometry::point::Point;

struct KdNode {
    split_pos: f32,
    split_axis: usize,
    right_child: usize,
    has_left_child: bool
}

impl KdNode {
    fn leaf() -> KdNode {
        KdNode {
            split_pos: 0.0,
            split_axis: 3,
            right_child: (1 << 29) - 1,
            has_left_child: false
        }
    }

    fn new(p: f32, a: usize) -> KdNode {
        KdNode {
            split_pos: p,
            split_axis: a,
            right_child: (1 << 29) - 1,
            has_left_child: false
        }
    }
}

pub trait KdTreeProc<NodeData> {
    fn run(&mut self, &Point, &NodeData, f32, &mut f32);
}

pub trait HasPoint {
    fn p<'a>(&'a self) -> &'a Point;
}

// !FIXME! Needs implementation!
pub struct KdTree<NodeData: HasPoint> {
    nodes: Vec<KdNode>,
    node_data: Vec<NodeData>
}

impl<NodeData: HasPoint> KdTree<NodeData> {
    pub fn lookup<U: KdTreeProc<NodeData>>(&self, m: &Point, p: &mut U, max_dist_sq: f32) {
        // Actually need a KdTree here...
        unimplemented!()
    }
}
