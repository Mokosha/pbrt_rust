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

    fn add_param(&mut self, name: &str, data: ParamTy) {
        let &mut ParamSet(ref mut map) = self;
        if let Some(_) = map.insert(name.to_string(), data) {
            println!("WARNING: Param {} already exists!", name);
        }
    }

    pub fn add_float(&mut self, name: &str, data: Vec<f32>) {
        self.add_param(name, ParamTy::Float(data))
    }

    pub fn find_one_float(&self, name: &str, def: f32) -> f32 {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Float(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0] }
        } else {
            def
        }
    }

    pub fn find_float<'a>(&'a self, name: &str) -> Option<&'a [f32]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Float(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_bool(&mut self, name: &str, data: Vec<bool>) {
        self.add_param(name, ParamTy::Bool(data))
    }

    pub fn find_one_bool(&self, name: &str, def: bool) -> bool {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Bool(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0] }
        } else {
            def
        }
    }

    pub fn find_bool<'a>(&'a self, name: &str) -> Option<&'a [bool]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Bool(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_int(&mut self, name: &str, data: Vec<i32>) {
        self.add_param(name, ParamTy::Int(data))
    }

    pub fn find_one_int(&self, name: &str, def: i32) -> i32 {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Int(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0] }
        } else {
            def
        }
    }

    pub fn find_int<'a>(&'a self, name: &str) -> Option<&'a [i32]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Int(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_point(&mut self, name: &str, data: Vec<Point>) {
        self.add_param(name, ParamTy::Point(data))
    }

    pub fn find_one_point(&self, name: &str, def: Point) -> Point {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Point(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_point<'a>(&'a self, name: &str) -> Option<&'a [Point]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Point(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_vec(&mut self, name: &str, data: Vec<Vector>) {
        self.add_param(name, ParamTy::Vec(data))
    }

    pub fn find_one_vec(&self, name: &str, def: Vector) -> Vector {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Vec(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_vec<'a>(&'a self, name: &str) -> Option<&'a [Vector]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Vec(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_normal(&mut self, name: &str, data: Vec<Normal>) {
        self.add_param(name, ParamTy::Normal(data))
    }

    pub fn find_one_normal(&self, name: &str, def: Normal) -> Normal {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Normal(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_normal<'a>(&'a self, name: &str) -> Option<&'a [Normal]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Normal(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_str(&mut self, name: &str, data: Vec<String>) {
        self.add_param(name, ParamTy::Str(data))
    }


    pub fn find_one_str(&self, name: &str, def: String) -> String {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Str(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_str<'a>(&'a self, name: &str) -> Option<&'a [String]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Str(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_tex(&mut self, name: &str, data: Vec<String>) {
        self.add_param(name, ParamTy::Tex(data))
    }


    pub fn find_one_tex(&self, name: &str, def: String) -> String {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Tex(ref f)) = map.get(name) {
            if f.is_empty() { def } else { f[0].clone() }
        } else {
            def
        }
    }

    pub fn find_tex<'a>(&'a self, name: &str) -> Option<&'a [String]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Tex(ref f) => Some(f.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_rgb_spectrum(&mut self, name: &str, data: Vec<f32>) {
        unimplemented!()
    }

    pub fn add_xyz_spectrum(&mut self, name: &str, data: Vec<f32>) {
        unimplemented!()
    }

    pub fn add_blackbody_spectrum(&mut self, name: &str, data: Vec<f32>) {
        unimplemented!()
    }

    pub fn find_one_spectrum(&self, name: &str, def: Spectrum) -> Spectrum {
        let &ParamSet(ref map) = self;
        if let Some(&ParamTy::Spec(ref s)) = map.get(name) {
            if s.is_empty() { def } else { s[0].clone() }
        } else { def }
    }

    pub fn find_spectrum<'a>(&'a self, name: &str)
                             -> Option<&'a [Spectrum]> {
        let &ParamSet(ref map) = self;
        map.get(name).and_then(|res| {
            match res {
                &ParamTy::Spec(ref s) => Some(s.as_slice()),
                _ => None
            }
        })
    }

    pub fn add_sampled_spectrum_files(&mut self, name: &str,
                                      data: Vec<String>) {
        unimplemented!()
    }

    pub fn add_sampled_spectrum(&mut self, name: &str, data: Vec<f32>) {
        unimplemented!()
    }
}

pub struct TextureParams<'a> {
    float_textures: Arc<HashMap<String, Arc<dyn Texture<f32>>>>,
    spectrum_textures: Arc<HashMap<String, Arc<dyn Texture<Spectrum>>>>,
    geom_params: &'a ParamSet,
    material_params: &'a ParamSet
}

fn find_texture<T>(textures: Arc<HashMap<String, Arc<dyn Texture<T>>>>,
                   name: &str) -> Arc<dyn Texture<T>> {
    match textures.get(name) {
        Some(tex) => tex.clone() as Arc<dyn Texture<T>>,
        None => panic!("Couldn't find texture named \"{}\"", name)
    }
}

impl<'a> TextureParams<'a> {
    pub fn new(geomp: &'a ParamSet, matp: &'a ParamSet,
               ft: Arc<HashMap<String, Arc<dyn Texture<f32>>>>,
               st: Arc<HashMap<String, Arc<dyn Texture<Spectrum>>>>)
               -> TextureParams<'a> {
        TextureParams {
            float_textures: ft,
            spectrum_textures: st,
            geom_params: geomp,
            material_params: matp
        }
    }

    pub fn get_spectrum_texture(&self, name: &str, def: &Spectrum)
                                -> Arc<dyn Texture<Spectrum>> {
        self.get_spectrum_texture_or_null(name).unwrap_or({
            let val = self.geom_params.find_one_spectrum(
                name, self.material_params.find_one_spectrum(name, def.clone()));
            Arc::new(ConstantTexture::new(val)) as Arc<dyn Texture<Spectrum>>
        })
    }

    pub fn get_spectrum_texture_or_null(&self, name: &str)
                                        -> Option<Arc<dyn Texture<Spectrum>>> {
        self.geom_params.find_tex(name).or(self.material_params.find_tex(name))
            .map(|names| find_texture(self.spectrum_textures.clone(), &names[0]))        
    }

    pub fn get_float_texture(&self, name: &str, def: f32)
                             -> Arc<dyn Texture<f32>> {
        self.get_float_texture_or_null(name).unwrap_or({
            let val = self.geom_params.find_one_float(
                name, self.material_params.find_one_float(name, def));
            Arc::new(ConstantTexture::new(val)) as Arc<dyn Texture<f32>>
        })
    }

    pub fn get_float_texture_or_null(&self, name: &str)
                                     -> Option<Arc<dyn Texture<f32>>> {
        self.geom_params.find_tex(name).or(self.material_params.find_tex(name))
            .map(|names| find_texture(self.float_textures.clone(), &names[0]))
    }

    pub fn find_float(&self, name: &str, def: f32) -> f32 {
        self.geom_params.find_one_float(
            name, self.material_params.find_one_float(name, def))
    }

    pub fn find_bool(&self, name: &str, def: bool) -> bool {
        self.geom_params.find_one_bool(
            name, self.material_params.find_one_bool(name, def))
    }

    pub fn find_int(&self, name: &str, def: i32) -> i32 {
        self.geom_params.find_one_int(
            name, self.material_params.find_one_int(name, def))
    }

    pub fn find_point(&self, name: &str, def: Point) -> Point {
        self.geom_params.find_one_point(
            name, self.material_params.find_one_point(name, def))
    }

    pub fn find_vec(&self, name: &str, def: Vector) -> Vector {
        self.geom_params.find_one_vec(
            name, self.material_params.find_one_vec(name, def))
    }

    pub fn find_normal(&self, name: &str, def: Normal) -> Normal {
        self.geom_params.find_one_normal(
            name, self.material_params.find_one_normal(name, def))
    }

    pub fn find_str(&self, name: &str, def: String) -> String {
        self.geom_params.find_one_str(
            name, self.material_params.find_one_str(name, def))
    }

    pub fn find_spectrum(&self, name: &str, def: Spectrum) -> Spectrum {
        self.geom_params.find_one_spectrum(
            name, self.material_params.find_one_spectrum(name, def))
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
