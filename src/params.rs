use std::collections::HashMap;

use geometry::normal::Normal;
use geometry::point::Point;
use geometry::vector::Vector;
use spectrum::Spectrum;

enum ParamTy {
    Bool(Vec<bool>),
    Int(Vec<i32>),
    Float(Vec<f32>),
    Point(Vec<Point>),
    Vec(Vec<Vector>),
    Normal(Vec<Normal>),
    Spec(Vec<Spectrum>),
    Str(Vec<String>),
    Tex(Vec<String>)
}

pub struct ParamSet(HashMap<String, ParamTy>);

impl ParamSet {
    pub fn new() -> ParamSet { ParamSet(HashMap::new()) }

    fn add_param(&mut self, name: &String, data: ParamTy) {
        let &mut ParamSet(ref mut map) = self;
        if let Some(_) = map.insert(name.clone(), data) {
            println!("WARNING: Param {} already exists!", name);
        }
    }

    pub fn add_float(&mut self, name: &String, data: Vec<f32>) {
        self.add_param(name, ParamTy::Float(data))
    }

    pub fn find_one_float(&self, name: &String, def: f32) -> f32 {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Float(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0] }
        } else {
            def
        }
    }

    pub fn add_bool(&mut self, name: &String, data: Vec<bool>) {
        self.add_param(name, ParamTy::Bool(data))
    }

    pub fn add_int(&mut self, name: &String, data: Vec<i32>) {
        self.add_param(name, ParamTy::Int(data))
    }

    pub fn add_point(&mut self, name: &String, data: Vec<Point>) {
        self.add_param(name, ParamTy::Point(data))
    }

    pub fn add_vec(&mut self, name: &String, data: Vec<Vector>) {
        self.add_param(name, ParamTy::Vec(data))
    }

    pub fn add_normal(&mut self, name: &String, data: Vec<Normal>) {
        self.add_param(name, ParamTy::Normal(data))
    }

    pub fn add_str(&mut self, name: &String, data: Vec<String>) {
        self.add_param(name, ParamTy::Str(data))
    }

    pub fn add_tex(&mut self, name: &String, data: Vec<String>) {
        self.add_param(name, ParamTy::Tex(data))
    }

    pub fn add_rgb_spectrum(&mut self, name: &String, data: Vec<f32>) {
        unimplemented!()
    }

    pub fn add_xyz_spectrum(&mut self, name: &String, data: Vec<f32>) {
        unimplemented!()
    }

    pub fn add_blackbody_spectrum(&mut self, name: &String, data: Vec<f32>) {
        unimplemented!()
    }

    pub fn add_sampled_spectrum_files(&mut self, name: &String, data: Vec<String>) {
        unimplemented!()
    }

    pub fn add_sampled_spectrum(&mut self, name: &String, data: Vec<f32>) {
        unimplemented!()
    }
}
