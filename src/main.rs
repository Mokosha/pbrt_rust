#[macro_use]
extern crate lazy_static;
extern crate pbrt_rust;

use pbrt_rust::scene::Scene;

struct Options {
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
}

const STATE_UNINITIALIZED: usize = 0;
const STATE_OPTIONS_BLOCK: usize = 1;
const STATE_WORLD_BLOCK: usize = 2;

lazy_static! {
    pub static ref PBRT_OPTIONS: Options = Options::new();
    static ref CURRENT_API_STATE: usize = STATE_UNINITIALIZED;
}

macro_rules! verify_initialized {
    ($x:expr) => {
        if *CURRENT_API_STATE == STATE_UNINITIALIZED {
            panic!("pbrt_init must be called before calling {}", $x);
        }
    };
}

fn parse_file(_ : &str) -> Option<Scene> { None }
fn pbrt_init(opts: &Options) {
    if *CURRENT_API_STATE != STATE_UNINITIALIZED {
        panic!("pbrt_init has already been called!");
    }
    *CURRENT_API_STATE = STATE_OPTIONS_BLOCK;

    *PBRT_OPTIONS = opts;
}

fn pbrt_cleanup() {
    if *CURRENT_API_STATE == STATE_UNINITIALIZED {
        panic!("pbrt_cleanup called before pbrt_init!");
    } else if *CURRENT_API_STATE == STATE_WORLD_BLOCK {
        panic!("pbrt_cleanup called inside world block!");
    }
    *CURRENT_API_STATE = STATE_UNINITIALIZED;
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
