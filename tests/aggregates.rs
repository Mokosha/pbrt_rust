#![allow(unused)]

extern crate pbrt_rust;

use pbrt_rust::bbox::BBox;
use pbrt_rust::bbox::HasBounds;
use pbrt_rust::primitive::Primitive;
use pbrt_rust::primitive::FullyRefinable;
use pbrt_rust::scene::Scene;

pub struct AggregateTester {
    num_iterations: usize,
    prims: Vec<Primitive>,
    bboxes: Vec<BBox>
}

impl AggregateTester {
    fn new(iters: usize, ps: Vec<Primitive>) -> AggregateTester {
        // Fully refine the aggregates...
        let refined = ps.into_iter().fold(Vec::new(), |mut ps, prim| {
            ps.append(&mut prim.fully_refine());
            ps
        });

        let bounds = refined.iter().map(|p| p.world_bound()).collect();
        
        AggregateTester {
            num_iterations: iters,
            prims: refined,
            bboxes: bounds
        }
    }

    // !FIXME! Once we have scenes to render, implement this from section
    // 4.6.1... For now the preliminary unit tests for aggregates will have
    // to do.
    fn render(scene: &Scene) {
        unimplemented!()
    }
}

#[ignore]
#[test]
fn render_scene_x() {
    // !FIXME! Find some actual scenes to test...
    unimplemented!()
}
