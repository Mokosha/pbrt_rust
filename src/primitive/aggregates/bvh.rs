use bbox::BBox;
use bbox::HasBounds;
use geometry::point::Point;
use primitive::Primitive;
use primitive::FullyRefinable;

#[derive(Clone, Debug, PartialEq, Copy)]
enum SplitMethod {
    Middle,
    EqualCounts,
    SAH
}

enum BVHNode {
    Empty,
    Leaf {
        bounds: BBox,
        first_prim_offset: usize,
        num_primitives: usize
    },
    Inner {
        bounds: BBox,
        children: [Box<BVHNode>; 2],
        split_axis: usize
    }
}

struct BVHPrimitiveInfo {
    primitive_number: usize,
    centroid: Point,
    bounds: BBox,
}

impl BVHPrimitiveInfo {
    fn new(pn: usize, bnds: BBox) -> BVHPrimitiveInfo {
        BVHPrimitiveInfo {
            primitive_number: pn,
            centroid: (&bnds.p_min + &bnds.p_max) * 0.5,
            bounds: bnds
        }
    }
}

#[derive(Clone, Debug)]
pub struct BVHAccel {
    max_prims_in_node: usize,
    split_method: SplitMethod,
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

        let build_data: Vec<_> = prims.iter().enumerate().map(|(i, p)| {
            let bbox = p.world_bound();
            BVHPrimitiveInfo::new(i, bbox)
        }).collect();

        // let (tree, ordered_prims) = recursive_build(build_data, prims);

        BVHAccel {
            max_prims_in_node: ::std::cmp::min(mp, 255),
            split_method: split_method,
            primitives: prims,
        }
    }
}
