#[macro_use]
extern crate lazy_static;
extern crate pbrt_rust;

use std::collections::HashMap;
use std::sync::Mutex;
use std::ops::Deref;
use std::ops::Index;
use std::ops::IndexMut;

use pbrt_rust::geometry::point::Point;
use pbrt_rust::geometry::vector::Vector;
use pbrt_rust::params::ParamSet;
use pbrt_rust::scene::Scene;
use pbrt_rust::transform::transform::Transform;

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
    camera_to_world: TransformSet
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
            camera_to_world: TransformSet::new()
        }
    }
}

lazy_static! {
    pub static ref PBRT_OPTIONS: Mutex<Options> = Mutex::new(Options::new());
    static ref CURRENT_API_STATE: Mutex<usize> = Mutex::new(STATE_UNINITIALIZED);

    static ref CUR_TRANSFORMS: Mutex<TransformSet> = Mutex::new(TransformSet::new());
    static ref ACTIVE_TRANSFORM_BITS: Mutex<usize> = Mutex::new(ALL_TRANSFORM_BITS);

    static ref NAMED_COORDINATE_SYSTEMS: Mutex<HashMap<String, TransformSet>> =
        Mutex::new(HashMap::new());

    static ref RENDER_OPTIONS: Mutex<RenderOptions> = Mutex::new(RenderOptions::new());
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
