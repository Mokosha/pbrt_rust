#[macro_use]
extern crate lazy_static;
extern crate pbrt_rust;

use pbrt_rust::scene::Scene;
use std::sync::Mutex;
use std::ops::Index;
use std::ops::IndexMut;

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

struct TransformSet {
    t: (Transform, Option<Transform>)
}

impl TransformSet {
    fn new() -> TransformSet {
        TransformSet { t: (Transform::new(), None) }
    }
}

impl Index<usize> for TransformSet {
    type Output = Transform;
    fn index(&self, index: usize) -> &Transform {
        match (index, &self.t) {
            (0, &(ref t, _)) => t,
            (1, &(_, Some(ref t))) => t,
            _ => panic!("Transform not available!")
        }
    }
}

impl IndexMut<usize> for TransformSet {
    fn index_mut(&mut self, index: usize) -> &mut Transform {
        match (index, &mut self.t) {
            (0, &mut (ref mut t, _)) => t,
            (1, &mut (_, Some(ref mut t))) => t,
            _ => panic!("Transform not available!")
        }
    }
}

lazy_static! {
    pub static ref PBRT_OPTIONS: Mutex<Options> = Mutex::new(Options::new());
    static ref CURRENT_API_STATE: Mutex<usize> = Mutex::new(STATE_UNINITIALIZED);

    static ref CUR_TRANSFORMS: Mutex<TransformSet> = Mutex::new(TransformSet::new());
    static ref ACTIVE_TRANSFORM_BITS: Mutex<usize> = Mutex::new(ALL_TRANSFORM_BITS);
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
