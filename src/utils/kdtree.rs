use bbox::*;
use geometry::point::Point;

use utils::partition_by;

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

impl HasPoint for Point {
    fn p<'a>(&'a self) -> &'a Point { self }
}

fn recursive_build<NodeData: HasPoint+Clone>(build_nodes: &mut [&NodeData],
                                             node_data: &mut Vec<NodeData>, nodes: &mut Vec<KdNode>) {
    // Create leaf node of kd-tree if we've reached the bottom
    if build_nodes.len() == 1 {
        nodes.push(KdNode::leaf());
        node_data.push(build_nodes[0].clone());
        return;
    }

    // Choose split direction and partition data
    let bound = build_nodes.iter().fold(BBox::new(), |bb, bn| {
        bb.unioned_with_ref(bn.p())
    });

    let split_axis = bound.max_extent();
    let split_pos = build_nodes.len() / 2;
    partition_by(build_nodes, |bn| bn.p()[split_axis]);

    // Allocate kd-tree node and continue recursively
    let node_num = nodes.len();
    assert_eq!(node_num, node_data.len());

    let new_node = KdNode::new(build_nodes[split_pos].p()[split_axis], split_axis);
    let new_data = build_nodes[split_pos].clone();

    nodes.push(new_node);
    node_data.push(new_data);

    let (left, right) = build_nodes.split_at_mut(split_pos);
    if left.len() > 0 {
        nodes[node_num].has_left_child = true;
        recursive_build(left, node_data, nodes);
    }

    if let Some((_, rest)) = right.split_first_mut() {
        if rest.len() > 0 {
            nodes[node_num].right_child = nodes.len();
            recursive_build(rest, node_data, nodes);
        }
    }
}

pub struct KdTree<NodeData: HasPoint+Clone> {
    nodes: Vec<KdNode>,
    node_data: Vec<NodeData>
}

impl<NodeData: HasPoint+Clone> KdTree<NodeData> {
    pub fn new(d: &Vec<NodeData>) -> KdTree<NodeData> {
        let num_nodes = d.len();
        let mut nodes = Vec::with_capacity(num_nodes);
        let mut data = Vec::with_capacity(num_nodes);

        let mut build_data: Vec<&NodeData> = d.iter().collect();
        recursive_build(&mut build_data, &mut data, &mut nodes);

        KdTree {
            nodes: nodes,
            node_data: data
        }
    }

    fn private_lookup<U: KdTreeProc<NodeData>>(&self, node_num: usize, m: &Point,
                                               p: &mut U, max_dist_sq: &mut f32) {
        let node = &self.nodes[node_num];

        // Process kd-tree node's children
        let axis = node.split_axis;
        if axis != 3 {
            let dist_sq = (m[axis] - node.split_pos) * (m[axis] - node.split_pos);
            let look_both = dist_sq <= *max_dist_sq;

            let on_left = m[axis] <= node.split_pos;
            let can_left = node.has_left_child;

            let on_right = !on_left;
            let can_right = node.right_child < self.nodes.len();

            let look_left = (look_both || on_left) && can_left;
            let look_right = (look_both || on_right) && can_right;

            if look_left {
                self.private_lookup(node_num + 1, m, p, max_dist_sq);
            }

            if look_right {
                self.private_lookup(node.right_child, m, p, max_dist_sq);
            }
        }

        // Hand kd-tree node to processing function
        let dist_sq = (self.node_data[node_num].p() - m).length_squared();
        if dist_sq <= *max_dist_sq {
            p.run(m, &self.node_data[node_num], dist_sq, max_dist_sq);
        }
    }

    pub fn lookup<U: KdTreeProc<NodeData>>(&self, m: &Point, p: &mut U,
                                           max_dist_sq: f32) {
        let mut mdsq = max_dist_sq;
        self.private_lookup(0, m, p, &mut mdsq);
    }
}
