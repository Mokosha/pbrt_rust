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
use volume_region::VolumeRegion;

pub struct Scene {
    aggregate : Primitive,
    lights : Vec<Light>,
    volume_region : Option<VolumeRegion>,
    // Scene Public data 23
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            aggregate: Primitive::geometric(Shape::sphere(
                Transform::new(), Transform::new(), false, 1.0, -1.0, 1.0, 360.0)),
            lights: vec![],
            volume_region: None,
        }
    }

    pub fn lights<'a>(&'a self) -> &'a Vec<Light> {
        &self.lights
    }

    // Scene Public methods 23
}

impl HasBounds for Scene {
    fn world_bound(&self) -> BBox {
        if let Some(volume) = self.volume_region {
            let agg_box = &(self.aggregate).world_bound();
            agg_box.union(&volume.world_bound())
        } else {
            self.aggregate.world_bound()
        }
    }
}

impl<'a> Intersectable<'a> for Scene {
    fn intersect(&'a self, ray : &Ray) -> Option<Intersection<'a>> {
        self.aggregate.intersect(ray)
    }

    fn intersect_p(&'a self, ray : &Ray) -> bool {
        self.aggregate.intersect_p(ray)
    }
}
