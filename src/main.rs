#[macro_use]
extern crate lazy_static;
extern crate pbrt_rust;

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::ops::Deref;
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
use pbrt_rust::scene::Scene;
use pbrt_rust::shape::Shape;
use pbrt_rust::transform::animated::AnimatedTransform;
use pbrt_rust::transform::transform::Transform;
use pbrt_rust::spectrum::Spectrum;
use pbrt_rust::texture::Texture;
use pbrt_rust::texture::ConstantTexture;

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
const START_TRANSFORM_BITS: usize = (1 << 0);
const END_TRANSFORM_BITS: usize = (1 << 1);
const ALL_TRANSFORM_BITS: usize = ((1 << MAX_TRANSFORMS) - 1);

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
            0...1 => &self.t[index],
            _ => panic!("Transform not available!")
        }
    }
}

impl IndexMut<usize> for TransformSet {
    fn index_mut(&mut self, index: usize) -> &mut Transform {
        match index {
            0...1 => &mut self.t[index],
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

    lights: Vec<Arc<Light>>,

    primitives: Vec<Arc<Primitive>>,

    instances: HashMap<String, Vec<Arc<Primitive>>>,
    current_instance: Option<String>
}

impl RenderOptions {
    fn new() -> RenderOptions {
        RenderOptions {
            transform_start_time: 0.0,
            transform_end_time: 0.0,

            filter_name: String::from("box"),
            filter_params: ParamSet::new(),

            film_name: String::new(),
            film_params: ParamSet::new(),

            sampler_name: String::new(),
            sampler_params: ParamSet::new(),

            accelerator_name: String::new(),
            accelerator_params: ParamSet::new(),

            surf_integrator_name: String::new(),
            surf_integrator_params: ParamSet::new(),

            vol_integrator_name: String::new(),
            vol_integrator_params: ParamSet::new(),

            renderer_name: String::new(),
            renderer_params: ParamSet::new(),

            camera_name: String::from("perspective"),
            camera_params: ParamSet::new(),
            camera_to_world: TransformSet::new(),

            lights: Vec::new(),
            primitives: Vec::new(),

            instances: HashMap::new(),
            current_instance: None,
        }
    }
}

#[derive(Clone, Debug)]
struct GraphicsState {
    material: String,
    material_params: ParamSet,
    float_textures: Arc<HashMap<String, Arc<Texture<f32>>>>,
    spectrum_textures: Arc<HashMap<String, Arc<Texture<Spectrum>>>>,

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

    fn create_material(&self, params: &ParamSet) -> Arc<Material> {
        let mp = TextureParams::new(params,
                                    &self.material_params,
                                    self.float_textures(),
                                    self.spectrum_textures());

        self.current_named_material.as_ref()
            .filter(|name| self.named_materials.contains_key(*name))
            .map_or(Arc::new(make_material(&self.material,
                                           &CUR_TRANSFORMS.lock().unwrap()[0],
                                           mp)),
                    |name| self.named_materials[name].clone())
    }

    fn float_textures(&self) -> Arc<HashMap<String, Arc<Texture<f32>>>> {
        self.float_textures.clone()
    }

    fn spectrum_textures(&self) -> Arc<HashMap<String, Arc<Texture<Spectrum>>>> {
        self.spectrum_textures.clone()
    }
}

lazy_static! {
    pub static ref PBRT_OPTIONS: Mutex<Options> =
        Mutex::new(Options::new());
    static ref CURRENT_API_STATE: Mutex<usize> =
        Mutex::new(STATE_UNINITIALIZED);

    static ref CUR_TRANSFORMS: Mutex<TransformSet> =
        Mutex::new(TransformSet::new());
    static ref ACTIVE_TRANSFORM_BITS: Mutex<usize> =
        Mutex::new(ALL_TRANSFORM_BITS);

    static ref NAMED_COORDINATE_SYSTEMS: Mutex<HashMap<String, TransformSet>> =
        Mutex::new(HashMap::new());

    static ref RENDER_OPTIONS: Mutex<RenderOptions> =
        Mutex::new(RenderOptions::new());
    static ref GRAPHICS_STATE: Mutex<GraphicsState> =
        Mutex::new(GraphicsState::new());

    static ref PUSHED_GRAPHICS_STATES: Mutex<Vec<GraphicsState>> =
        Mutex::new(Vec::new());
    static ref PUSHED_TRANSFORMS: Mutex<Vec<TransformSet>> =
        Mutex::new(Vec::new());
    static ref PUSHED_ACTIVE_TRANSFORM_BITS: Mutex<Vec<usize>> =
        Mutex::new(Vec::new());
}

fn for_active_transforms<T: Fn(&mut Transform)>(f: T) {
    for i in 0..MAX_TRANSFORMS {
        if ((1 << i) & *(ACTIVE_TRANSFORM_BITS.lock().unwrap())) != 0 {
            f(&mut CUR_TRANSFORMS.lock().unwrap()[i]);
        }
    }
}

fn get_current_api_state() -> usize {
    *(CURRENT_API_STATE.lock().unwrap())
}

fn set_current_api_state(x: usize) {
    *(CURRENT_API_STATE.lock().unwrap()) = x;
}

macro_rules! verify_initialized {
    ($x:expr) => {
        if get_current_api_state() == STATE_UNINITIALIZED {
            panic!("pbrt_init must be called before calling {}", $x);
        }
    };
}

macro_rules! verify_options {
    ($x:expr) => {
        if get_current_api_state() != STATE_OPTIONS_BLOCK {
            panic!("{} must be called from an options block!", $x);
        }
    };
}

macro_rules! verify_world {
    ($x:expr) => {
        if get_current_api_state() != STATE_WORLD_BLOCK {
            panic!("{} must be called from a world block!", $x);
        }
    };
}

macro_rules! warn_if_animated_xform {
    ($x:expr) => {
        if CUR_TRANSFORMS.lock().unwrap().is_animated() {
            print!("Animated transformations set; ignoring for {} and ", $x);
            println!("using the start transform only");
        }
    };
}

fn pbrt_attribute_begin() {
    verify_world!("AttributeBegin");
    PUSHED_GRAPHICS_STATES.lock().unwrap().push(
        GRAPHICS_STATE.lock().unwrap().clone());
    PUSHED_TRANSFORMS.lock().unwrap().push(
        CUR_TRANSFORMS.lock().unwrap().clone());
    PUSHED_ACTIVE_TRANSFORM_BITS.lock().unwrap().push(
        ACTIVE_TRANSFORM_BITS.lock().unwrap().clone());
}

fn pbrt_attribute_end() {
    verify_world!("AttributeEnd");
    if let Some(bits) = PUSHED_ACTIVE_TRANSFORM_BITS.lock().unwrap().pop() {
        *(ACTIVE_TRANSFORM_BITS.lock().unwrap()) = bits;
        *(CUR_TRANSFORMS.lock().unwrap()) =
            PUSHED_TRANSFORMS.lock().unwrap().pop().unwrap();
        *(GRAPHICS_STATE.lock().unwrap()) =
            PUSHED_GRAPHICS_STATES.lock().unwrap().pop().unwrap();
    } else {
        println!("WARNING: Unmatched pbrt_attribute_end encountered. Ignoring.")
    }
}

fn pbrt_transform_begin() {
    verify_world!("TransformBegin");
    PUSHED_TRANSFORMS.lock().unwrap().push(
        CUR_TRANSFORMS.lock().unwrap().clone());
}

fn pbrt_transform_end() {
    verify_world!("TransformEnd");
    if let Some(xf) = PUSHED_TRANSFORMS.lock().unwrap().pop() {
        *(CUR_TRANSFORMS.lock().unwrap()) = xf;
    } else {
        println!("WARNING: Unmatched pbrt_transform_end encountered. Ignoring.")
    }
}

fn pbrt_identity() {
    verify_initialized!("Identity");
    for_active_transforms(|t| {
        *t = Transform::new();
    });
}

fn pbrt_translate(dx: f32, dy: f32, dz: f32) {
    verify_initialized!("Translate");
    for_active_transforms(|t| {
        *t = t.clone() * Transform::translate(&Vector::new_with(dx, dy, dz));
    });
}

fn pbrt_rotate(angle: f32, ax: f32, ay: f32, az: f32) {
    verify_initialized!("Rotate");
    for_active_transforms(|t| {
        *t = t.clone() * Transform::rotate(angle, &Vector::new_with(ax, ay, az));
    });
}

fn pbrt_scale(sx: f32, sy: f32, sz: f32) {
    verify_initialized!("Scale");
    for_active_transforms(|t| {
        *t = t.clone() * Transform::scale(sx, sy, sz);
    });
}

fn pbrt_lookat(ex: f32, ey: f32, ez: f32,
               lx: f32, ly: f32, lz: f32,
               ux: f32, uy: f32, uz: f32) {
    verify_initialized!("Look At");
    for_active_transforms(|t| {
        *t = t.clone() * Transform::look_at(
            &Point::new_with(ex, ey, ez),
            &Point::new_with(lx, ly, lz),
            &Vector::new_with(ux, uy, uz));
    });
}

fn pbrt_concat_transform(xf: [f32; 16]) {
    verify_initialized!("Concat");
    for_active_transforms(|t| {
        *t = t.clone() * Transform::from([
            [xf[0], xf[1], xf[2], xf[3]],
            [xf[4], xf[5], xf[6], xf[7]],
            [xf[8], xf[9], xf[10], xf[11]],
            [xf[12], xf[13], xf[14], xf[15]]]);
    });
}

fn pbrt_transform(xf: [f32; 16]) {
    verify_initialized!("Transform");
    for_active_transforms(|t| {
        *t = Transform::from([
            [xf[0], xf[1], xf[2], xf[3]],
            [xf[4], xf[5], xf[6], xf[7]],
            [xf[8], xf[9], xf[10], xf[11]],
            [xf[12], xf[13], xf[14], xf[15]]]);
    });
}

fn pbrt_coordinate_system(name: String) {
    verify_initialized!("CoordinateSystem");
    NAMED_COORDINATE_SYSTEMS.lock().unwrap()
        .insert(name, CUR_TRANSFORMS.lock().unwrap().clone());
}

fn pbrt_coord_sys_transform(name: String) {
    verify_initialized!("CoordSysTransform");
    if let Some(t) = NAMED_COORDINATE_SYSTEMS.lock().unwrap().get(&name) {
        *(CUR_TRANSFORMS.lock().unwrap()) = t.clone();
    } else {
        println!("WARNING: No coordinate system named {}", name);
    }
}

fn pbrt_active_transform_all() {
    *(ACTIVE_TRANSFORM_BITS.lock().unwrap()) = ALL_TRANSFORM_BITS;
}

fn pbrt_active_transform_end_time() {
    *(ACTIVE_TRANSFORM_BITS.lock().unwrap()) = END_TRANSFORM_BITS;
}

fn pbrt_active_transform_start_time() {
    *(ACTIVE_TRANSFORM_BITS.lock().unwrap()) = START_TRANSFORM_BITS;
}

fn pbrt_transform_times(start: f32, end: f32) {
    verify_options!("TransformTimes");
    RENDER_OPTIONS.lock().unwrap().transform_start_time = start;
    RENDER_OPTIONS.lock().unwrap().transform_end_time = end;
}

fn pbrt_pixel_filter(name: &String, params: &ParamSet) {
    verify_options!("PixelFilter");
    RENDER_OPTIONS.lock().unwrap().filter_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().filter_params = params.clone();
}

fn pbrt_sampler(name: &String, params: &ParamSet) {
    verify_options!("Sampler");
    RENDER_OPTIONS.lock().unwrap().sampler_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().sampler_params = params.clone();
}

fn pbrt_accelerator(name: &String, params: &ParamSet) {
    verify_options!("Accelerator");
    RENDER_OPTIONS.lock().unwrap().accelerator_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().accelerator_params = params.clone();
}

fn pbrt_surf_integrator(name: &String, params: &ParamSet) {
    verify_options!("SurfaceIntegrator");
    RENDER_OPTIONS.lock().unwrap().surf_integrator_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().surf_integrator_params = params.clone();
}

fn pbrt_vol_integrator(name: &String, params: &ParamSet) {
    verify_options!("VolumeIntegrator");
    RENDER_OPTIONS.lock().unwrap().vol_integrator_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().vol_integrator_params = params.clone();
}

fn pbrt_renderer(name: &String, params: &ParamSet) {
    verify_options!("Renderer");
    RENDER_OPTIONS.lock().unwrap().renderer_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().renderer_params = params.clone();
}

fn pbrt_camera(name: &String, params: &ParamSet) {
    verify_options!("Camera");
    RENDER_OPTIONS.lock().unwrap().camera_name = name.clone();
    RENDER_OPTIONS.lock().unwrap().camera_params = params.clone();
    RENDER_OPTIONS.lock().unwrap().camera_to_world =
        inverse(CUR_TRANSFORMS.lock().unwrap().deref());
    NAMED_COORDINATE_SYSTEMS.lock().unwrap().insert(
        String::from("camera"), RENDER_OPTIONS.lock().unwrap().camera_to_world.clone());
}

fn make_float_texture(name: &String, tex_to_world: &Transform, params: TextureParams) -> Arc<Texture<f32>> {
    match name.as_ref() {
        "constant" => Arc::new(ConstantTexture::new(params.find_float(&("value".to_string()), 0.0))),
        _ => panic!("Unknown float texture type: {}", name),
    }
}

fn make_color_texture(name: &String, tex_to_world: &Transform, params: TextureParams) -> Arc<Texture<Spectrum>> {
    match name.as_ref() {
        "definitely isn't this" => panic!("Oops, it is."),
        _ => panic!("Unknown color texture type: {}", name),
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

fn make_light(name: &str, light_to_world: &Transform, params: &ParamSet) -> Arc<Light> {
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

fn make_accelerator(name: &str, prims: &Vec<Arc<Primitive>>, params: &ParamSet) -> Vec<Arc<Primitive>> {
    unimplemented!()
}

fn pbrt_make_named_material(name: &String, params: &ParamSet) {
    verify_world!("make_named_material");
    let mtl_params = GRAPHICS_STATE.lock().unwrap().material_params.clone();
    let mp = TextureParams::new(params, &mtl_params,
                                GRAPHICS_STATE.lock().unwrap().float_textures(),
                                GRAPHICS_STATE.lock().unwrap().spectrum_textures());
    let mat_name = mp.find_str("type", "unknown_type".to_string());
    warn_if_animated_xform!("make_named_material");
    if let "" = mat_name.as_ref() {
        panic!("No parameter string \"type\" found in make_named_material");
    } else {
        let mtl = make_material(&mat_name, &CUR_TRANSFORMS.lock().unwrap()[0], mp);
        GRAPHICS_STATE.lock().unwrap().named_materials.insert(name.clone(), Arc::new(mtl));
    }
}

fn pbrt_material(name: &String, params: &ParamSet) {
    verify_world!("Material");
    GRAPHICS_STATE.lock().unwrap().material = name.clone();
    GRAPHICS_STATE.lock().unwrap().material_params = params.clone();
    GRAPHICS_STATE.lock().unwrap().current_named_material = None;
}

fn pbrt_texture(name: &String, ty: &String, texname: &String, params: &ParamSet) {
    verify_world!("Texture");
    let mut fts = GRAPHICS_STATE.lock().unwrap().float_textures();
    let mut sts = GRAPHICS_STATE.lock().unwrap().spectrum_textures();
    let tp = TextureParams::new(params, params, fts.clone(), sts.clone());
    match ty.as_ref() {
        "float" => {
            if fts.contains_key("float") {
                println!("Texture {} being redefined", texname);
            }
            warn_if_animated_xform!("Texture");
            let ft = make_float_texture(
                texname, &CUR_TRANSFORMS.lock().unwrap()[0], tp);
            (*Arc::get_mut(&mut fts).unwrap()).insert(name.clone(), ft);
        },
        "color" => {
            if fts.contains_key("color") {
                println!("Texture {} being redefined", texname);
            }
            warn_if_animated_xform!("Texture");
            let st = make_color_texture(
                texname, &CUR_TRANSFORMS.lock().unwrap()[0], tp);
            (*Arc::get_mut(&mut sts).unwrap()).insert(name.clone(), st);
        },
        _ => panic!("Texture type {} unknown!", ty),
    }
}

fn pbrt_light_source(name: &String, params: &ParamSet) {
    verify_world!("LightSource");
    warn_if_animated_xform!("LightSource");
    let lt = make_light(name, &CUR_TRANSFORMS.lock().unwrap()[0], params);
    RENDER_OPTIONS.lock().unwrap().lights.push(lt);
}

fn pbrt_area_light_source(name: &String, params: &ParamSet) {
    verify_world!("AreaLightSource");
    GRAPHICS_STATE.lock().unwrap().area_light = name.clone();
    GRAPHICS_STATE.lock().unwrap().area_light_params = params.clone();
}

fn pbrt_shape(name: &String, params: &ParamSet) {
    verify_world!("Shape");
    let ro = GRAPHICS_STATE.lock().unwrap().reverse_orientation;
    let prim =
        // Create primitive for animated shape
        if CUR_TRANSFORMS.lock().unwrap().is_animated() {
            // Create initial shape for animated shape
            if !GRAPHICS_STATE.lock().unwrap().area_light.is_empty() {
                println!("Warning: ignoring currently set area light when creating animated shape");
            }

            let id = Transform::new();
            let shape = make_shape(name, id.clone(), id.clone(), ro, params);
            let mtl = GRAPHICS_STATE.lock().unwrap().create_material(params);

            // Get animated world_to_object transform for shape
            let w2o0 = CUR_TRANSFORMS.lock().unwrap()[0].clone();
            let w2o1 = CUR_TRANSFORMS.lock().unwrap()[1].clone();
            let xf_start = RENDER_OPTIONS.lock().unwrap().transform_start_time;
            let xf_end = RENDER_OPTIONS.lock().unwrap().transform_end_time;

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
                let t = CUR_TRANSFORMS.lock().unwrap()[0].clone();
                let t_inv = t.inverse();
                (t, t_inv)
            };
            let shape =
                make_shape(name, obj_to_world.clone(), world_to_obj, ro, params);
            let mtl = GRAPHICS_STATE.lock().unwrap().create_material(params);

            // Possibly create area light for shape
            if !GRAPHICS_STATE.lock().unwrap().area_light.is_empty() {
                let area_light =
                    make_area_light(
                        &GRAPHICS_STATE.lock().unwrap().area_light,
                        &obj_to_world,
                        &GRAPHICS_STATE.lock().unwrap().area_light_params,
                        shape.clone());

                Primitive::geometric_area_light(shape, mtl, Arc::new(area_light))
            } else {
                Primitive::geometric(shape, mtl)
            }
        };
    let prim = Arc::new(prim);

    // Add primitive to scene or current instance
    if let Some(i) = RENDER_OPTIONS.lock().unwrap().current_instance.as_ref() {
        RENDER_OPTIONS.lock().unwrap().instances.get_mut(i).unwrap().push(prim)
    } else {
        if let Some(light) = prim.area_light() {
            RENDER_OPTIONS.lock().unwrap().lights.push(light);
        }
        RENDER_OPTIONS.lock().unwrap().primitives.push(prim);
    }
}

fn pbrt_object_begin(name: String) {
    verify_world!("ObjectBegin");
    pbrt_attribute_begin();
    if RENDER_OPTIONS.lock().unwrap().current_instance.is_some() {
        panic!("ObjectBegin called inside of instance definition!");
    }

    RENDER_OPTIONS.lock().unwrap().instances.insert(name.clone(), Vec::new());
    RENDER_OPTIONS.lock().unwrap().current_instance = Some(name);
}

fn pbrt_object_end() {
    verify_world!("ObjectEnd");
    if RENDER_OPTIONS.lock().unwrap().current_instance.is_some() {
        panic!("ObjectEnd called outside of instance definition!");
    }

    RENDER_OPTIONS.lock().unwrap().current_instance = None;
    pbrt_attribute_end();
}

fn pbrt_object_instance(name: &String) {
    verify_world!("ObjectInstance");
    if !RENDER_OPTIONS.lock().unwrap().instances.contains_key(name) {
        println!("Can't find object named {}", name);
    }

    if RENDER_OPTIONS.lock().unwrap().instances.get(name).unwrap().is_empty() {
        return;
    }

    if RENDER_OPTIONS.lock().unwrap().instances.get(name).map_or(false, |prims| {
        prims.len() > 1 || !prims[0].can_intersect()
    }) {
        // Refine instance Primitives and create aggregate
        let accel = make_accelerator(&RENDER_OPTIONS.lock().unwrap().accelerator_name,
                                     RENDER_OPTIONS.lock().unwrap().instances.get(name).unwrap(),
                                     &RENDER_OPTIONS.lock().unwrap().accelerator_params);
        RENDER_OPTIONS.lock().unwrap().instances.insert(name.to_string(), accel);
    }

    let w2i0 = CUR_TRANSFORMS.lock().unwrap()[0].clone();
    let w2i1 = CUR_TRANSFORMS.lock().unwrap()[1].clone();
    let xf_start = RENDER_OPTIONS.lock().unwrap().transform_start_time;
    let xf_end = RENDER_OPTIONS.lock().unwrap().transform_end_time;
    let animated_world_to_instance =
        AnimatedTransform::new(w2i0, xf_start, w2i1, xf_end);
    let prim = RENDER_OPTIONS.lock().unwrap().instances.get(name).unwrap()[0].clone();
    RENDER_OPTIONS.lock().unwrap().primitives.push(prim);    
}

fn pbrt_world_begin() {
    verify_options!("WorldBegin");
    set_current_api_state(STATE_WORLD_BLOCK);
    pbrt_active_transform_all();
    for_active_transforms(|t| { *t = Transform::new(); });
    NAMED_COORDINATE_SYSTEMS.lock().unwrap().insert(
        String::from("world"), CUR_TRANSFORMS.lock().unwrap().clone());
}

fn parse_file(_ : &str) -> Option<Scene> { None }
fn pbrt_init(opts: &Options) {
    if get_current_api_state() != STATE_UNINITIALIZED {
        panic!("pbrt_init has already been called!");
    }
    set_current_api_state(STATE_OPTIONS_BLOCK);

    PBRT_OPTIONS.lock().unwrap().copy_from(opts);
}

fn pbrt_cleanup() {
    if get_current_api_state() != STATE_UNINITIALIZED {
        panic!("pbrt_cleanup called before pbrt_init!");
    } else if get_current_api_state() == STATE_WORLD_BLOCK {
        panic!("pbrt_cleanup called inside world block!");
    }
    set_current_api_state(STATE_UNINITIALIZED);
}

fn main() {
    let options = Options::new();
    let filenames : Vec<String> = vec![];
    // Process command line arguments
    pbrt_init(&options);
    if filenames.len() == 0 {
        parse_file("-");
    } else {
        for filename in &filenames {
            if let Some(_) = parse_file(&filename) {
            } else {
                panic!("Cannot open scene file \"{}\"", filename);
            }
        }
    }
    pbrt_cleanup();
}
