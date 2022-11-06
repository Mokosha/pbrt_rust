extern crate pbrt_rust;

use std::collections::HashMap;
use std::sync::Arc;
use std::ops::Index;
use std::ops::IndexMut;

use pbrt_rust::area_light::AreaLight;
use pbrt_rust::geometry::point::Point;
use pbrt_rust::geometry::vector::Vector;
use pbrt_rust::material::Material;
use pbrt_rust::light::point::PointLight;
use pbrt_rust::light::Light;
use pbrt_rust::params::{ParamSet, TextureParams};
use pbrt_rust::primitive::{Primitive, FullyRefinable};
use pbrt_rust::renderer::Renderer;
use pbrt_rust::scene::Scene;
use pbrt_rust::shape::Shape;
use pbrt_rust::spectrum::Spectrum;
use pbrt_rust::texture::Texture;
use pbrt_rust::texture::ConstantTexture;
use pbrt_rust::transform::animated::AnimatedTransform;
use pbrt_rust::transform::transform::Transform;
use pbrt_rust::volume::VolumeRegion;
use pbrt_rust::volume::aggregate::AggregateVolumeRegion;

pub struct Options {
    num_cores: usize,
    quick_render: bool,
    quiet: bool,
    verbose: bool,
    open_window: bool,
    image_file: String
}

impl Options {
    fn new() -> Options {
        Options {
            num_cores: 0,
            quick_render: false,
            quiet: false,
            verbose: false,
            open_window: false,
            image_file: String::new()
        }
    }

    fn copy_from(&mut self, other: &Options) {
        self.num_cores = other.num_cores;
        self.quick_render = other.quick_render;
        self.quiet = other.quiet;
        self.verbose = other.verbose;
        self.open_window = other.open_window;
        self.image_file = other.image_file.clone();
    }
}

const STATE_UNINITIALIZED: usize = 0;
const STATE_OPTIONS_BLOCK: usize = 1;
const STATE_WORLD_BLOCK: usize = 2;

const MAX_TRANSFORMS: usize = 2;
const START_TRANSFORM_BITS: usize = 1 << 0;
const END_TRANSFORM_BITS: usize = 1 << 1;
const ALL_TRANSFORM_BITS: usize = (1 << MAX_TRANSFORMS) - 1;

#[derive(Clone, Debug, PartialEq)]
struct TransformSet {
    t: Vec<Transform>
}

impl TransformSet {
    fn new() -> TransformSet {
        TransformSet { t: vec![Transform::new(), Transform::new()] }
    }

    fn is_animated(&self) -> bool {
        self.t.iter().zip(self.t.iter().skip(1)).any(|(t1, t2)| t1 != t2)
    }
}

fn inverse(ts: &TransformSet) -> TransformSet {
    let mut t2 = ts.clone();
    for t in t2.t.iter_mut() {
        *t = t.inverse();
    }
    t2
}

impl Index<usize> for TransformSet {
    type Output = Transform;
    fn index(&self, index: usize) -> &Transform {
        match index {
            0..=1 => &self.t[index],
            _ => panic!("Transform not available!")
        }
    }
}

impl Index<usize> for &TransformSet {
    type Output = Transform;
    fn index(&self, index: usize) -> &Transform {
        match index {
            0..=1 => &self.t[index],
            _ => panic!("Transform not available!")
        }
    }
}

impl IndexMut<usize> for TransformSet {
    fn index_mut(&mut self, index: usize) -> &mut Transform {
        match index {
            0..=1 => &mut self.t[index],
            _ => panic!("Transform not available!")
        }
    }
}

#[derive(Debug)]
pub struct RenderOptions {
    transform_start_time: f32,
    transform_end_time: f32,

    filter_name: String,
    filter_params: ParamSet,

    film_name: String,
    film_params: ParamSet,

    sampler_name: String,
    sampler_params: ParamSet,

    accelerator_name: String,
    accelerator_params: ParamSet,

    surf_integrator_name: String,
    surf_integrator_params: ParamSet,

    vol_integrator_name: String,
    vol_integrator_params: ParamSet,

    renderer_name: String,
    renderer_params: ParamSet,

    camera_name: String,
    camera_params: ParamSet,
    camera_to_world: TransformSet,

    lights: Vec<Arc<dyn Light>>,

    primitives: Vec<Primitive>,

    instances: HashMap<String, Vec<Primitive>>,
    current_instance: Option<String>,

    volume_regions: Vec<Arc<dyn VolumeRegion>>,
}

impl RenderOptions {
    fn new() -> RenderOptions {
        RenderOptions {
            transform_start_time: 0.0,
            transform_end_time: 1.0,

            filter_name: String::from("box"),
            filter_params: ParamSet::new(),

            film_name: String::from("image"),
            film_params: ParamSet::new(),

            sampler_name: String::from("lowdiscrepancy"),
            sampler_params: ParamSet::new(),

            accelerator_name: String::from("bvh"),
            accelerator_params: ParamSet::new(),

            surf_integrator_name: String::new(),
            surf_integrator_params: ParamSet::new(),

            vol_integrator_name: String::new(),
            vol_integrator_params: ParamSet::new(),

            renderer_name: String::from("sampler"),
            renderer_params: ParamSet::new(),

            camera_name: String::from("perspective"),
            camera_params: ParamSet::new(),
            camera_to_world: TransformSet::new(),

            lights: Vec::new(),
            primitives: Vec::new(),

            instances: HashMap::new(),
            current_instance: None,

            volume_regions: Vec::new(),
        }
    }

    fn make_renderer(&self) -> Arc<dyn Renderer> {
        unimplemented!()
    }

    fn make_scene(&mut self) -> Scene {
        // initialize volume region
        let volume_region = {
            if self.volume_regions.is_empty() { None }
            else if self.volume_regions.len() == 1 { Some(self.volume_regions[0].clone()) }
            else {
                let b: Arc<dyn VolumeRegion> =
                    Arc::new(AggregateVolumeRegion::new(self.volume_regions.clone()));
                Some(b)
            }
        };

        let accelerator = make_accelerator(&self.accelerator_name,
                                           &self.primitives,
                                           &self.accelerator_params);

        let scene = Scene::new_with(
            Arc::new(accelerator),
            self.lights.clone(),
            volume_region);

        // Erase primitives lights and volume regions from render options
        self.primitives.clear();
        self.lights.clear();
        self.volume_regions.clear();

        scene
    }
}

#[derive(Clone, Debug)]
struct GraphicsState {
    material: String,
    material_params: ParamSet,
    float_textures: Arc<HashMap<String, Arc<dyn Texture<f32>>>>,
    spectrum_textures: Arc<HashMap<String, Arc<dyn Texture<Spectrum>>>>,

    named_materials: HashMap<String, Arc<Material>>,
    current_named_material: Option<String>,

    area_light: String,
    area_light_params: ParamSet,

    reverse_orientation: bool,
}

impl GraphicsState {
    fn new() -> GraphicsState {
        GraphicsState {
            material: String::from("matte"),
            material_params: ParamSet::new(),
            float_textures: Arc::new(HashMap::new()),
            spectrum_textures: Arc::new(HashMap::new()),
            named_materials: HashMap::new(),
            current_named_material: None,
            area_light: String::new(),
            area_light_params: ParamSet::new(),
            reverse_orientation: false,
        }
    }

    fn create_material(&self, tex_to_world: &Transform, params: &ParamSet) -> Arc<Material> {
        let mp = TextureParams::new(params,
                                    &self.material_params,
                                    self.float_textures(),
                                    self.spectrum_textures());

        self.current_named_material.as_ref()
            .filter(|name| self.named_materials.contains_key(*name))
            .map_or(Arc::new(make_material(&self.material,
                                           tex_to_world,
                                           mp)),
                    |name| self.named_materials[name].clone())
    }

    fn float_textures(&self) -> Arc<HashMap<String, Arc<dyn Texture<f32>>>> {
        self.float_textures.clone()
    }

    fn spectrum_textures(&self) -> Arc<HashMap<String, Arc<dyn Texture<Spectrum>>>> {
        self.spectrum_textures.clone()
    }
}

fn make_material(name: &String, _tex_to_world: &Transform, params: TextureParams) -> Material {
    match name.as_ref() {
        "matte" => Material::matte(
            params.get_spectrum_texture("Kd", &Spectrum::from(0.5)),
            params.get_float_texture("sigma", 0.0),
            params.get_float_texture_or_null("bumpmap")),
        _ => panic!("Unknown material type: {}", name),
    }
}

fn make_light(name: &str, light_to_world: &Transform, params: &ParamSet) -> Arc<dyn Light> {
    match name {
        "point" => {
            let i = params.find_one_spectrum("I", Spectrum::from(1.0));
            let sc = params.find_one_spectrum("scale", Spectrum::from(1.0));
            let p = params.find_one_point("from", Point::new_with(0.0, 0.0, 0.0));
            let l2w = Transform::translate(&Vector::new_with(p.x, p.y, p.z)) * light_to_world;
            Arc::new(PointLight::new(l2w, i * sc))
        },
        _ => panic!("Unknown light type: {}", name)
    }
}

fn make_area_light(name: &str, light_to_world: &Transform, params: &ParamSet,
                   shape: Shape) -> AreaLight {
    unimplemented!()
}

fn make_shape(name: &str, obj_to_world: Transform, world_to_obj: Transform,
              reverse_orientation: bool, params: &ParamSet) -> Shape {
    match name {
        "sphere" => {
            let radius = params.find_one_float("radius", 1.0);
            let zmin = params.find_one_float("zmin", -radius);
            let zmax = params.find_one_float("zmax", radius);
            let phimax = params.find_one_float("phimax", 360.0);
            Shape::sphere(obj_to_world, world_to_obj, reverse_orientation,
                          radius, zmin, zmax, phimax)
        },
        _ => panic!("Unknown shape type: {}", name)
    }
}

fn make_accelerator(name: &str, prims: &Vec<Primitive>, params: &ParamSet) -> Primitive {
    unimplemented!()
}

struct Pbrt {
  options: Options,
  current_api_state: usize,
  current_transforms: TransformSet,
  active_transform_bits: usize,
  named_coordinate_systems: HashMap<String, TransformSet>,
  render_options: RenderOptions,
  graphics_state: GraphicsState,
  pushed_graphics_states: Vec<GraphicsState>,
  pushed_transforms: Vec<TransformSet>,
  pushed_active_transform_bits: Vec<usize>
}

macro_rules! verify_initialized {
    ($self:ident, $x:expr) => {
        if $self.get_current_api_state() == STATE_UNINITIALIZED {
            panic!("pbrt_init must be called before calling {}", $x);
        }
    };
}

macro_rules! verify_options {
    ($self:ident, $x:expr) => {
        if $self.get_current_api_state() != STATE_OPTIONS_BLOCK {
            panic!("{} must be called from an options block!", $x);
        }
    };
}

macro_rules! verify_world {
    ($self:ident, $x:expr) => {
        if $self.get_current_api_state() != STATE_WORLD_BLOCK {
            panic!("{} must be called from a world block!", $x);
        }
    };
}

macro_rules! warn_if_animated_xform {
    ($self:ident, $x:expr) => {
        if $self.current_transforms.is_animated() {
            print!("Animated transformations set; ignoring for {} and ", $x);
            println!("using the start transform only");
        }
    };
}

impl Pbrt {
    fn for_active_transforms<T: Fn(&mut Transform)>(&mut self, f: T) {
        for i in 0..MAX_TRANSFORMS {
            if ((1 << i) & self.active_transform_bits) != 0 {
                f(&mut self.current_transforms[i]);
            }
        }
    }

    fn get_current_api_state(&self) -> usize { self.current_api_state }

    fn set_current_api_state(&mut self, x: usize) { self.current_api_state = x; }

    fn attribute_begin(&mut self) {
        verify_world!(self, "AttributeBegin");
        self.pushed_graphics_states.push(self.graphics_state.clone());
        self.pushed_transforms.push(self.current_transforms.clone());
        self.pushed_active_transform_bits.push(self.active_transform_bits.clone());
    }

    fn attribute_end(&mut self) {
        verify_world!(self, "AttributeEnd");
        if let Some(bits) = self.pushed_active_transform_bits.pop() {
            self.active_transform_bits = bits;
            self.current_transforms = self.pushed_transforms.pop().unwrap();
            self.graphics_state = self.pushed_graphics_states.pop().unwrap();
        } else {
            println!("WARNING: Unmatched pbrt_attribute_end encountered. Ignoring.")
        }
    }

    fn transform_begin(&mut self) {
        verify_world!(self, "TransformBegin");
        self.pushed_transforms.push(self.current_transforms.clone());
    }

    fn transform_end(&mut self) {
        verify_world!(self, "TransformEnd");
        if let Some(xf) = self.pushed_transforms.pop() {
            self.current_transforms = xf;
        } else {
            println!("WARNING: Unmatched pbrt_transform_end encountered. Ignoring.")
        }
    }

    fn identity(&mut self) {
        verify_initialized!(self, "Identity");
        self.for_active_transforms(|t| {
            *t = Transform::new();
        });
    }
    
    fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        verify_initialized!(self, "Translate");
        self.for_active_transforms(|t| {
            *t = t.clone() * Transform::translate(&Vector::new_with(dx, dy, dz));
        });
    }

    fn rotate(&mut self, angle: f32, ax: f32, ay: f32, az: f32) {
        verify_initialized!(self, "Rotate");
        self.for_active_transforms(|t| {
            *t = t.clone() * Transform::rotate(angle, &Vector::new_with(ax, ay, az));
        });
    }

    fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
        verify_initialized!(self, "Scale");
        self.for_active_transforms(|t| {
            *t = t.clone() * Transform::scale(sx, sy, sz);
        });
    }
    
    fn lookat(&mut self,
                   ex: f32, ey: f32, ez: f32,
                   lx: f32, ly: f32, lz: f32,
                   ux: f32, uy: f32, uz: f32) {
        verify_initialized!(self, "Look At");
        self.for_active_transforms(|t| {
            *t = t.clone() * Transform::look_at(
                &Point::new_with(ex, ey, ez),
                &Point::new_with(lx, ly, lz),
                &Vector::new_with(ux, uy, uz));
        });
    }
    
    fn concat_transform(&mut self, xf: [f32; 16]) {
        verify_initialized!(self, "Concat");
        self.for_active_transforms(|t| {
            *t = t.clone() * Transform::from([
                [xf[0], xf[1], xf[2], xf[3]],
                [xf[4], xf[5], xf[6], xf[7]],
                [xf[8], xf[9], xf[10], xf[11]],
                [xf[12], xf[13], xf[14], xf[15]]]);
        });
    }
    
    fn transform(&mut self, xf: [f32; 16]) {
        verify_initialized!(self, "Transform");
        self.for_active_transforms(|t| {
            *t = Transform::from([
                [xf[0], xf[1], xf[2], xf[3]],
                [xf[4], xf[5], xf[6], xf[7]],
                [xf[8], xf[9], xf[10], xf[11]],
                [xf[12], xf[13], xf[14], xf[15]]]);
        });
    }

    fn coordinate_system(&mut self, name: String) {
        verify_initialized!(self, "CoordinateSystem");
        self.named_coordinate_systems.insert(name, self.current_transforms.clone());
    }

    fn coord_sys_transform(&mut self, name: String) {
        verify_initialized!(self, "CoordSysTransform");
        if let Some(t) = self.named_coordinate_systems.get(&name) {
            self.current_transforms = t.clone();
        } else {
            println!("WARNING: No coordinate system named {}", name);
        }
    }

    fn active_transform_all(&mut self) {
        self.active_transform_bits = ALL_TRANSFORM_BITS;
    }

    fn active_transform_end_time(&mut self) {
        self.active_transform_bits = END_TRANSFORM_BITS;
    }

    fn active_transform_start_time(&mut self) {
        self.active_transform_bits = START_TRANSFORM_BITS;
    }

    fn transform_times(&mut self, start: f32, end: f32) {
        verify_options!(self, "TransformTimes");
        self.render_options.transform_start_time = start;
        self.render_options.transform_end_time = end;
    }

    fn pixel_filter(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "PixelFilter");
        self.render_options.filter_name = name.clone();
        self.render_options.filter_params = params.clone();
    }

    fn sampler(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "Sampler");
        self.render_options.sampler_name = name.clone();
        self.render_options.sampler_params = params.clone();
    }

    fn accelerator(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "Accelerator");
        self.render_options.accelerator_name = name.clone();
        self.render_options.accelerator_params = params.clone();
    }

    fn surf_integrator(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "SurfaceIntegrator");
        self.render_options.surf_integrator_name = name.clone();
        self.render_options.surf_integrator_params = params.clone();
    }

    fn vol_integrator(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "VolumeIntegrator");
        self.render_options.vol_integrator_name = name.clone();
        self.render_options.vol_integrator_params = params.clone();
    }

    fn renderer(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "Renderer");
        self.render_options.renderer_name = name.clone();
        self.render_options.renderer_params = params.clone();
    }

    fn camera(&mut self, name: &String, params: &ParamSet) {
        verify_options!(self, "Camera");
        self.render_options.camera_name = name.clone();
        self.render_options.camera_params = params.clone();
        self.render_options.camera_to_world = inverse(&self.current_transforms);
        self.named_coordinate_systems.insert(
            String::from("camera"), self.render_options.camera_to_world.clone());
    }

    fn make_named_material(&mut self, name: &String, params: &ParamSet) {
        verify_world!(self, "make_named_material");
        let gs = &self.graphics_state;
        let mtl_params = gs.material_params.clone();
        let mp = TextureParams::new(params, &mtl_params,
                                    gs.float_textures(),
                                    gs.spectrum_textures());
        let mat_name = mp.find_str("type", "unknown_type".to_string());
        warn_if_animated_xform!(self, "make_named_material");
        if let "" = mat_name.as_ref() {
            panic!("No parameter string \"type\" found in make_named_material");
        } else {
            let mtl = make_material(&mat_name, &self.current_transforms[0], mp);
            self.graphics_state.named_materials.insert(name.clone(), Arc::new(mtl));
        }
    }

    fn material(&mut self, name: &String, params: &ParamSet) {
        verify_world!(self, "Material");
        self.graphics_state.material = name.clone();
        self.graphics_state.material_params = params.clone();
        self.graphics_state.current_named_material = None;
    }

    fn texture(&mut self, name: &String, ty: &String, texname: &String, params: &ParamSet) {
        verify_world!(self, "Texture");
        let mut fts = self.graphics_state.float_textures();
        let mut sts = self.graphics_state.spectrum_textures();
        let tp = TextureParams::new(params, params, fts.clone(), sts.clone());
        match ty.as_ref() {
            "float" => {
                if fts.contains_key("float") {
                    println!("Texture {} being redefined", texname);
                }
                warn_if_animated_xform!(self, "Texture");
                let ft = match texname.as_str() {
                    "constant" => ConstantTexture::new(params.find_one_float(&("value".to_string()), 0.0)),
                    _ => panic!("Unknown float texture type: {}", texname),
                };
                (*Arc::get_mut(&mut fts).unwrap()).insert(texname.clone(), Arc::new(ft));
            },
            "color" => {
                if fts.contains_key("color") {
                    println!("Texture {} being redefined", texname);
                }
                warn_if_animated_xform!(self, "Texture");
                let st = match texname.as_ref() {
                    "definitely isn't this" => panic!("Oops, it is."),
                    _ => panic!("Unknown color texture type: {}", texname),
                };
                (*Arc::get_mut(&mut sts).unwrap()).insert(name.clone(), st);
            },
            _ => panic!("Texture type {} unknown!", ty),
        }
    }

    fn light_source(&mut self, name: &String, params: &ParamSet) {
        verify_world!(self, "LightSource");
        warn_if_animated_xform!(self, "LightSource");
        let lt = make_light(name, &self.current_transforms[0], params);
        self.render_options.lights.push(lt);
    }

    fn area_light_source(&mut self, name: &String, params: &ParamSet) {
        verify_world!(self, "AreaLightSource");
        self.graphics_state.area_light = name.clone();
        self.graphics_state.area_light_params = params.clone();
    }

    fn shape(&mut self, name: &String, params: &ParamSet) {
        verify_world!(self, "Shape");
        let ro = self.graphics_state.reverse_orientation;
        let prim =
            // Create primitive for animated shape
            if self.current_transforms.is_animated() {
                // Create initial shape for animated shape
                if !self.graphics_state.area_light.is_empty() {
                    println!("Warning: ignoring currently set area light when creating animated shape");
                }
    
                let id = Transform::new();
                let shape = make_shape(name, id.clone(), id.clone(), ro, params);
                let mtl = self.graphics_state.create_material(&self.current_transforms[0], params);
    
                // Get animated world_to_object transform for shape
                let w2o0 = self.current_transforms[0].clone();
                let w2o1 = self.current_transforms[1].clone();
                let xf_start = self.render_options.transform_start_time;
                let xf_end = self.render_options.transform_end_time;
    
                let animated_world_to_object =
                    AnimatedTransform::new(w2o0, xf_start, w2o1, xf_end);
    
                if !shape.can_intersect() {
                    // Refine animated shape and create BVH if more than one shape
                    // created
                    let base_prim = Primitive::geometric(shape, mtl);
                    let refined_prims = base_prim.fully_refine();
                    if refined_prims.is_empty() { return; }
                    if refined_prims.len() > 1 {
                        let bvh = Primitive::bvh(refined_prims, 10, "equal");
                        Primitive::transformed(Arc::new(bvh), animated_world_to_object)
                    } else {
                        Primitive::transformed(
                            Arc::new(refined_prims.into_iter().last().unwrap()),
                            animated_world_to_object)
                    }
                } else {
                    Primitive::transformed(Arc::new(Primitive::geometric(shape, mtl)),
                                           animated_world_to_object)
                }
            } else {
                // Create primitive for static shape
                let (obj_to_world, world_to_obj) = {
                    let t = self.current_transforms[0].clone();
                    let t_inv = t.inverse();
                    (t, t_inv)
                };
                let shape =
                    make_shape(name, obj_to_world.clone(), world_to_obj, ro, params);
                let mtl = self.graphics_state.create_material(&self.current_transforms[0], params);
    
                // Possibly create area light for shape
                if !self.graphics_state.area_light.is_empty() {
                    let area_light =
                        make_area_light(
                            &self.graphics_state.area_light,
                            &obj_to_world,
                            &self.graphics_state.area_light_params,
                            shape.clone());
    
                    Primitive::geometric_area_light(shape, mtl, Arc::new(area_light))
                } else {
                    Primitive::geometric(shape, mtl)
                }
            };
    
        // Add primitive to scene or current instance
        if let Some(i) = self.render_options.current_instance.as_ref() {
            self.render_options.instances.get_mut(i).unwrap().push(prim)
        } else {
            if let Some(light) = prim.area_light() {
                self.render_options.lights.push(light);
            }
            self.render_options.primitives.push(prim);
        }
    }

    fn object_begin(&mut self, name: String) {
        verify_world!(self, "ObjectBegin");
        self.attribute_begin();
        if self.render_options.current_instance.is_some() {
            panic!("ObjectBegin called inside of instance definition!");
        }
    
        self.render_options.instances.insert(name.clone(), Vec::new());
        self.render_options.current_instance = Some(name);
    }

    fn object_end(&mut self) {
        verify_world!(self, "ObjectEnd");
        if self.render_options.current_instance.is_some() {
            panic!("ObjectEnd called outside of instance definition!");
        }
    
        self.render_options.current_instance = None;
        self.attribute_end();
    }

    fn object_instance(&mut self, name: &String) {
        verify_world!(self, "ObjectInstance");
        if !self.render_options.instances.contains_key(name) {
            println!("Can't find object named {}", name);
        }
    
        if self.render_options.instances.get(name).unwrap().is_empty() {
            return;
        }
    
        if self.render_options.instances.get(name).map_or(false, |prims| {
            prims.len() > 1 || !prims[0].can_intersect()
        }) {
            // Refine instance Primitives and create aggregate
            let accel = make_accelerator(&self.render_options.accelerator_name,
                                         self.render_options.instances.get(name).unwrap(),
                                         &self.render_options.accelerator_params);
            self.render_options.instances.insert(name.to_string(), vec![accel]);
        }
    
        let w2i0 = self.current_transforms[0].clone();
        let w2i1 = self.current_transforms[1].clone();
        let xf_start = self.render_options.transform_start_time;
        let xf_end = self.render_options.transform_end_time;
        let animated_world_to_instance =
            AnimatedTransform::new(w2i0, xf_start, w2i1, xf_end);
        let prim = self.render_options.instances.get(name).unwrap()[0].clone();
        self.render_options.primitives.push(prim);    
    }

    fn world_begin(&mut self) {
        verify_options!(self, "WorldBegin");
        self.set_current_api_state(STATE_WORLD_BLOCK);
        self.active_transform_all();
        self.for_active_transforms(|t| { *t = Transform::new(); });
        self.named_coordinate_systems.insert(
            String::from("world"), self.current_transforms.clone());
    }

    fn world_end(&mut self) {
        verify_world!(self, "WorldEnd");
        // Ensure there are no pushed graphics states
        while self.pushed_graphics_states.len() > 0 {
            println!("Missing end to pbrt_attribute_begin()");
            self.pushed_graphics_states.pop();
            self.pushed_transforms.pop();
        }
    
        while self.pushed_transforms.len() > 0 {
            println!("Missing end to pbrt_transform_begin()");
            self.pushed_transforms.pop();
        }
    
        // Create scene and render
        let mut renderer = self.render_options.make_renderer();
        let scene = self.render_options.make_scene();
        Arc::get_mut(&mut renderer).unwrap().render(&scene);
    
        // Clean up after rendering
        self.set_current_api_state(STATE_OPTIONS_BLOCK);
        for i in 0..MAX_TRANSFORMS {
            self.current_transforms[i] = Transform::new();
        }
        self.active_transform_all();
        self.named_coordinate_systems.clear();
    }

    fn parse_file(&mut self, _ : &str) -> Option<Scene> { None }

    fn init(opts: Options) -> Pbrt {
        Pbrt {
            options: opts,
            current_api_state: STATE_OPTIONS_BLOCK,
            current_transforms: TransformSet::new(),
            active_transform_bits: ALL_TRANSFORM_BITS,
            named_coordinate_systems: HashMap::new(),
            render_options: RenderOptions::new(),
            graphics_state: GraphicsState::new(),
            pushed_graphics_states: Vec::new(),
            pushed_active_transform_bits: Vec::new(),
            pushed_transforms: Vec::new()
        }
    }
    
    fn cleanup(&mut self) {
        if self.get_current_api_state() != STATE_UNINITIALIZED {
            panic!("pbrt_cleanup called before pbrt_init!");
        } else if self.get_current_api_state() == STATE_WORLD_BLOCK {
            panic!("pbrt_cleanup called inside world block!");
        }
        self.set_current_api_state(STATE_UNINITIALIZED);
    }

    pub fn run(opts: Options, filenames: Vec<String>) {
        let mut pbrt = Pbrt::init(opts);
        if filenames.len() == 0 {
            pbrt.parse_file("-");
        } else {
            for filename in &filenames {
                if let Some(_) = pbrt.parse_file(&filename) {
                } else {
                    panic!("Cannot open scene file \"{}\"", filename);
                }
            }
        }
        pbrt.cleanup();
    }
}

fn main() {
    let options = Options::new();
    let filenames : Vec<String> = vec![];
    // Process command line arguments
    Pbrt::run(options, filenames);
}
