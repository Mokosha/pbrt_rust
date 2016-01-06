use bbox;
use bbox::Union;
use bbox::HasBounds;
use intersection;
use light;
use primitive;
use ray;
use volume_region;

pub struct Scene {
    pub aggregate : primitive::Primitive,
    pub lights : Vec<light::Light>,
    pub volume_region : Option<volume_region::VolumeRegion>,

    pub world_bound : bbox::BBox,
    // Scene Public data 23
}

impl bbox::HasBounds for Scene {
    fn world_bound(&self) -> bbox::BBox {
        if let Some(volume) = self.volume_region {
            let agg_box = &(self.aggregate).world_bound();
            (&agg_box).union(&volume.world_bound())
        } else {
            self.aggregate.world_bound()
        }
    }
}

impl Scene {
    pub fn new() -> Scene {
        let scene = Scene {
            aggregate: primitive::Primitive,
            lights: vec![],
            volume_region: None,
            world_bound: bbox::BBox::new()
        };

        let world_bound = scene.world_bound();
        scene
    }

    pub fn intersect(&self, ray : &ray::Ray,
                 isect : &mut intersection::Intersection) -> bool {
        self.aggregate.intersect(ray, isect)
    }

    pub fn intersect_p(&self, ray : &ray::Ray) -> bool {
        self.aggregate.intersect_p(ray)
    }
    // Scene Public methods 23
}
