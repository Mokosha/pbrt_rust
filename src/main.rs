extern crate pbrt_rust;
use pbrt_rust::scene::Scene;

struct Options;

impl Options {
    fn new() -> Options { Options }
}

fn parse_file(_ : &str) -> Option<Scene> { None }
fn pbrt_init(_ : &Options) { }
fn pbrt_cleanup() { }

fn main() {
    let options = Options::new();
    let filenames : Vec<String> = vec![];
    // Process command line arguments
    pbrt_init(&options);
    if filenames.len() == 0 {
        parse_file("-");
    } else {
        for filename in &filenames {
            if let Some(scene) = parse_file(&filename) {
            } else {
                panic!("Cannot open scene file \"{}\"", filename);
            }
        }
    }
    pbrt_cleanup();
}
