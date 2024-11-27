use bbox::BBox;
use bbox::Union;
use bbox::HasBounds;
use intersection::Intersectable;
use intersection::Intersection;
use light::Light;
use primitive::Primitive;
use ray::Ray;
use shape::Shape;
use transform::transform::Transform;
use volume::VolumeRegion;

use std::sync::Arc;

pub struct Scene {
    aggregate : Arc<Primitive>,
    lights : Vec<Arc<dyn Light>>,
    volume_region : Option<Arc<dyn VolumeRegion>>,
    // Scene Public data 23
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            aggregate: Arc::new(Primitive::simple(Shape::sphere(
                Transform::new(), Transform::new(), false, 1.0, -1.0, 1.0, 360.0))),
            lights: vec![],
            volume_region: None,
        }
    }

    pub fn new_with(aggregate: Arc<Primitive>,
                    lights: Vec<Arc<dyn Light>>,
                    volume_region: Option<Arc<dyn VolumeRegion>>) -> Scene {
        Scene {
            aggregate: aggregate.clone(),
            lights: lights.clone(),
            volume_region: volume_region
        }
    }

    pub fn lights(&self) -> Vec<Arc<dyn Light>> {
        self.lights.clone()
    }

    // Scene Public methods 23
}

impl HasBounds for Scene {
    fn world_bound(&self) -> BBox {
        if let Some(ref volume) = self.volume_region {
            let agg_box = &(self.aggregate).world_bound();
            agg_box.union(&volume.world_bound())
        } else {
            self.aggregate.world_bound()
        }
    }
}

impl Intersectable for Scene {
    fn intersect(&self, ray : &Ray) -> Option<Intersection> {
        self.aggregate.intersect(ray)
    }

    fn intersect_p(&self, ray : &Ray) -> bool {
        self.aggregate.intersect_p(ray)
    }
}
