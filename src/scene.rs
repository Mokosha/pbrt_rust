use bbox::BBox;
use bbox::Union;
use bbox::HasBounds;
use intersection::Intersectable;
use intersection::Intersection;
use light::Light;
use primitive::Primitive;
use primitive::GeometricPrimitive;
use ray;
use volume_region;

pub struct SceneBase<Prim: Primitive> {
    aggregate : Prim,
    lights : Vec<Light>,
    volume_region : Option<volume_region::VolumeRegion>,
    // Scene Public data 23
}

impl<Prim: Primitive> SceneBase<Prim> {
    pub fn new() -> SceneBase<Prim> {
        SceneBase {
            aggregate: Prim::new(),
            lights: vec![],
            volume_region: None,
        }
    }
}

pub enum Scene {
    GeometricPrimitiveScene(SceneBase<GeometricPrimitive>)
}

impl HasBounds for Scene {
    fn world_bound(&self) -> BBox {
        let base = match self {
            &Scene::GeometricPrimitiveScene(ref b) => b
        };

        if let Some(volume) = base.volume_region {
            let agg_box = &(base.aggregate).world_bound();
            agg_box.union(&volume.world_bound())
        } else {
            base.aggregate.world_bound()
        }
    }
}

impl<'a> Intersectable<'a> for Scene {
    fn intersect(&'a self, ray : &ray::Ray) -> Option<Intersection<'a>> {
        match self {
            &Scene::GeometricPrimitiveScene(ref b) => b.aggregate.intersect(ray)
        }
    }

    fn intersect_p(&'a self, ray : &ray::Ray) -> bool {
        match self {
            &Scene::GeometricPrimitiveScene(ref b) => b.aggregate.intersect_p(ray)
        }
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene::GeometricPrimitiveScene(SceneBase::<GeometricPrimitive>::new())
    }
    
    pub fn lights<'a>(&'a self) -> &'a Vec<Light> {
        match self {
            &Scene::GeometricPrimitiveScene(ref b) => &b.lights
        }
    }
    
    // Scene Public methods 23
}
