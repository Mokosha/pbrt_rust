use primitive::Primitive;
use primitive::FullyRefinable;

#[derive(Clone, Debug, PartialEq, Copy)]
enum SplitMethod {
    Middle,
    EqualCounts,
    SAH
}

#[derive(Clone, Debug)]
pub struct BVHAccel {
    max_prims_in_node: usize,
    split_method: SplitMethod,
    primitives: Vec<Primitive>
}

impl BVHAccel {
    fn new(p: Vec<Primitive>, mp: usize, sm: &'static str) -> BVHAccel {
        let prims = p.into_iter().fold(Vec::new(), |mut ps, prim| {
            ps.append(&mut prim.fully_refine());
            ps
        });

        BVHAccel {
            max_prims_in_node: ::std::cmp::min(mp, 255),
            split_method: match sm {
                "sah" => SplitMethod::SAH,
                "middle" => SplitMethod::Middle,
                "equal" => SplitMethod::EqualCounts,
                _ => {
                    println!("Warning: BVH split method {} unknown. Using \"SAH\"", sm);
                    SplitMethod::SAH
                },
            },
            primitives: prims
        }
    }
}
