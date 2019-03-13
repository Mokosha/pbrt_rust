use std::collections::HashMap;
use std::sync::Arc;

use geometry::normal::Normal;
use geometry::point::Point;
use geometry::vector::Vector;
use spectrum::Spectrum;
use texture::ConstantTexture;
use texture::Texture;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

    pub fn find_float<'a>(&'a self, name: &String) -> Option<&'a [f32]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Float(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_bool(&mut self, name: &String, data: Vec<bool>) {
        self.add_param(name, ParamTy::Bool(data))
    }

    pub fn find_one_bool(&self, name: &String, def: bool) -> bool {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Bool(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0] }
        } else {
            def
        }
    }

    pub fn find_bool<'a>(&'a self, name: &String) -> Option<&'a [bool]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Bool(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_int(&mut self, name: &String, data: Vec<i32>) {
        self.add_param(name, ParamTy::Int(data))
    }

    pub fn find_one_int(&self, name: &String, def: i32) -> i32 {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Int(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0] }
        } else {
            def
        }
    }

    pub fn find_int<'a>(&'a self, name: &String) -> Option<&'a [i32]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Int(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_point(&mut self, name: &String, data: Vec<Point>) {
        self.add_param(name, ParamTy::Point(data))
    }

    pub fn find_one_point(&self, name: &String, def: Point) -> Point {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Point(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_point<'a>(&'a self, name: &String) -> Option<&'a [Point]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Point(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_vec(&mut self, name: &String, data: Vec<Vector>) {
        self.add_param(name, ParamTy::Vec(data))
    }

    pub fn find_one_vec(&self, name: &String, def: Vector) -> Vector {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Vec(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_vec<'a>(&'a self, name: &String) -> Option<&'a [Vector]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Vec(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_normal(&mut self, name: &String, data: Vec<Normal>) {
        self.add_param(name, ParamTy::Normal(data))
    }

    pub fn find_one_normal(&self, name: &String, def: Normal) -> Normal {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Normal(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_normal<'a>(&'a self, name: &String) -> Option<&'a [Normal]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Normal(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_str(&mut self, name: &String, data: Vec<String>) {
        self.add_param(name, ParamTy::Str(data))
    }


    pub fn find_one_str(&self, name: &String, def: String) -> String {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Str(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_str<'a>(&'a self, name: &String) -> Option<&'a [String]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Str(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_tex(&mut self, name: &String, data: Vec<String>) {
        self.add_param(name, ParamTy::Tex(data))
    }


    pub fn find_one_tex(&self, name: &String, def: String) -> String {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Tex(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_tex<'a>(&'a self, name: &String) -> Option<&'a [String]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Tex(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
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

    pub fn find_one_spectrum(&self, name: &String, def: Spectrum) -> Spectrum {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Spec(ref s)) = map.get(name) {
            if s.is_empty() { def } else { s[0].clone() }
        } else { def }
    }

    pub fn find_spectrum<'a>(&'a self, name: &String)
                             -> Option<&'a [Spectrum]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Spec(ref s) => Some(s.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_sampled_spectrum_files(&mut self, name: &String,
                                      data: Vec<String>) {
        unimplemented!()
    }

    pub fn add_sampled_spectrum(&mut self, name: &String, data: Vec<f32>) {
        unimplemented!()
    }
}

pub struct TextureParams<'a> {
    float_textures: Arc<HashMap<String, Arc<Texture<f32>>>>,
    spectrum_textures: Arc<HashMap<String, Arc<Texture<Spectrum>>>>,
    geom_params: &'a ParamSet,
    material_params: &'a ParamSet
}

fn find_texture<T>(textures: Arc<HashMap<String, Arc<Texture<T>>>>,
                   name: &String) -> Arc<Texture<T>> {
    match textures.get(name) {
        Some(tex) => tex.clone() as Arc<Texture<T>>,
        None => panic!("Couldn't find texture named \"{}\"", name)
    }
}

impl<'a> TextureParams<'a> {
    pub fn new(geomp: &'a ParamSet, matp: &'a ParamSet,
               ft: Arc<HashMap<String, Arc<Texture<f32>>>>,
               st: Arc<HashMap<String, Arc<Texture<Spectrum>>>>)
               -> TextureParams<'a> {
        TextureParams {
            float_textures: ft,
            spectrum_textures: st,
            geom_params: geomp,
            material_params: matp
        }
    }

    pub fn get_spectrum_texture(&self, n: &String, def: &Spectrum)
                                -> Arc<Texture<Spectrum>> {
        if let Some(names) = self.geom_params.find_tex(n) {
            find_texture(self.spectrum_textures.clone(), &names[0])
        } else if let Some(names) = self.material_params.find_tex(n) {
            find_texture(self.spectrum_textures.clone(), &names[0])
        } else {
            let val = self.geom_params.find_one_spectrum(
                n, self.material_params.find_one_spectrum(n, def.clone()));
            Arc::new(ConstantTexture::new(val)) as Arc<Texture<Spectrum>>
        }
    }

    pub fn get_float_texture(&self, n: &String, def: f32)
                                -> Arc<Texture<f32>> {
        if let Some(names) = self.geom_params.find_tex(n) {
            find_texture(self.float_textures.clone(), &names[0])
        } else if let Some(names) = self.material_params.find_tex(n) {
            find_texture(self.float_textures.clone(), &names[0])
        } else {
            let val = self.geom_params.find_one_float(
                n, self.material_params.find_one_float(n, def));
            Arc::new(ConstantTexture::new(val)) as Arc<Texture<f32>>
        }
    }

    pub fn find_float(&self, n: &String, def: f32) -> f32 {
        self.geom_params.find_one_float(
            n, self.material_params.find_one_float(n, def))
    }

    pub fn find_bool(&self, n: &String, def: bool) -> bool {
        self.geom_params.find_one_bool(
            n, self.material_params.find_one_bool(n, def))
    }

    pub fn find_int(&self, n: &String, def: i32) -> i32 {
        self.geom_params.find_one_int(
            n, self.material_params.find_one_int(n, def))
    }

    pub fn find_point(&self, n: &String, def: Point) -> Point {
        self.geom_params.find_one_point(
            n, self.material_params.find_one_point(n, def))
    }

    pub fn find_vec(&self, n: &String, def: Vector) -> Vector {
        self.geom_params.find_one_vec(
            n, self.material_params.find_one_vec(n, def))
    }

    pub fn find_normal(&self, n: &String, def: Normal) -> Normal {
        self.geom_params.find_one_normal(
            n, self.material_params.find_one_normal(n, def))
    }

    pub fn find_str(&self, n: &String, def: String) -> String {
        self.geom_params.find_one_str(
            n, self.material_params.find_one_str(n, def))
    }

    pub fn find_spectrum(&self, n: &String, def: Spectrum) -> Spectrum {
        self.geom_params.find_one_spectrum(
            n, self.material_params.find_one_spectrum(n, def))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_be_created() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_add_params() {
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_lookup_single_params() {
        // Make sure defaults work, too.
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn it_can_lookup_multiple_params() {
        unimplemented!()
    }
}
