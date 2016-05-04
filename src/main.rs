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

macro_rules! verify_initialized {
    ($x:expr) => {
        if *api_state == STATE_UNINITIALIZED {
            panic!("pbrt_init must be called before calling {}", $x);
        }
    };
}

fn parse_file(_ : &str) -> Option<Scene> { None }
fn pbrt_init(api_state: &mut usize, _ : &Options) {
    if *api_state != STATE_UNINITIALIZED {
        panic!("pbrt_init has already been called!");
    }
    *api_state = STATE_OPTIONS_BLOCK;
}

fn pbrt_cleanup(api_state: &mut usize) {
    if *api_state == STATE_UNINITIALIZED {
        panic!("pbrt_cleanup called before pbrt_init!");
    } else if *api_state == STATE_WORLD_BLOCK {
        panic!("pbrt_cleanup called inside world block!");
    }
    *api_state = STATE_UNINITIALIZED;
}

fn main() {
    let options = Options::new();
    let filenames : Vec<String> = vec![];
    // Process command line arguments
    let mut g_current_api_state: usize = STATE_UNINITIALIZED;
    pbrt_init(&mut g_current_api_state, &options);
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
    pbrt_cleanup(&mut g_current_api_state);
}
