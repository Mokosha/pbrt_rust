extern crate pbrt_rust;

use pbrt_rust::renderer::Renderer;
use pbrt_rust::scene;
use pbrt_rust::sampler_renderer::SamplerRenderer;

struct Options;

impl Options {
    fn new() -> Options { Options }
}

fn parse_file(filename : &str) -> Option<scene::Scene> { None }
fn pbrt_init(options : &Options) { }
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
