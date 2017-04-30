extern crate lazy_static;
extern crate image;

use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::Arc;

use self::image::open;
use self::image::ImageResult;

use spectrum::Spectrum;
use texture::mapping2d::TextureMapping2D;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
enum ImageWrap { }

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct MIPMap<T> {
    width: u32,
    height: u32,
    pixels: Vec<T>
}

impl<T> MIPMap<T> {
    pub fn new(w: u32, h: u32, pixels: Vec<T>) -> MIPMap<T> {
        MIPMap { width: w, height: h, pixels: pixels }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct TexInfo {
    filename: String,
    do_trilinear: bool,
    max_aniso: f32,
    wrap: ImageWrap,
    gamma: f32
}

impl Eq for TexInfo {}
impl Ord for TexInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut result = self.filename.cmp(&other.filename);
        if result != Ordering::Equal {
            return result;
        }

        result = self.do_trilinear.cmp(&other.do_trilinear);
        if result != Ordering::Equal {
            return result;
        }

        if self.max_aniso < other.max_aniso {
            return Ordering::Less;
        } else if self.max_aniso > other.max_aniso {
            return Ordering::Greater;
        }

        if self.gamma < other.gamma {
            return Ordering::Less;
        } else if self.gamma > other.gamma {
            return Ordering::Greater;
        }        

        self.wrap.cmp(&other.wrap)
    }
}

type TextureCache<T> = BTreeMap<TexInfo, Arc<MIPMap<T>>>;

lazy_static! {
    static ref FLOAT_TEXTURES: Mutex<TextureCache<f32>> =
        Mutex::new(TextureCache::new());
    static ref SPECTRUM_TEXTURES: Mutex<TextureCache<Spectrum>> =
        Mutex::new(TextureCache::new());
}

struct ImageTexture<Tmemory> {
    mipmap: Arc<MIPMap<Tmemory>>,
    mapping: Box<TextureMapping2D>
}

fn read_image(filename: &String) -> ImageResult<(u32, u32, Vec<Spectrum>)> {
    open(filename)
        .and_then(|raw_img| Ok(raw_img.to_rgb()))
        .and_then(|rgb_img| {
            let mut spec_vec = Vec::new();
            for &pixel in rgb_img.pixels() {
                let r = (pixel.data[0] as f32) / 255f32;
                let g = (pixel.data[1] as f32) / 255f32;
                let b = (pixel.data[2] as f32) / 255f32;
                spec_vec.push(Spectrum::from_rgb([r, g, b]));
            }
            Ok((rgb_img.width(), rgb_img.height(), spec_vec))
        })
}

// !TODO! Use function specialization here once it becomes stable
// https://github.com/rust-lang/rust/issues/31844
fn get_f32_texture(filename: &String, do_trilinear: bool,
                   max_aniso: f32, wrap_mode: ImageWrap, scale: f32,
                   gamma: f32) -> Arc<MIPMap<f32>> {
    let tex_info = TexInfo {
        filename: filename.clone(),
        do_trilinear: do_trilinear,
        max_aniso: max_aniso,
        wrap: wrap_mode,
        gamma: gamma
    };

    if let Some(tex) = FLOAT_TEXTURES.lock().unwrap().get(&tex_info) {
        return tex.clone();
    }

    let read_img_result = read_image(filename);
    let ret = if let Ok((width, height, texels)) = read_img_result {
        // Convert texels to f32 and create MIPMap
        let pixels = texels.into_iter()
            .map(|s| (s.y() * scale).powf(gamma))
            .collect();
        Arc::new(MIPMap::new(width, height, pixels))
    } else {
        // Create one-values mipmap
        Arc::new(MIPMap::new(1, 1, vec![scale.powf(gamma); 1]))
    };

    FLOAT_TEXTURES.lock().unwrap().insert(tex_info.clone(), ret);
    FLOAT_TEXTURES.lock().unwrap().get(&tex_info).unwrap().clone()
}

fn get_spectrum_texture(filename: &String, do_trilinear: bool,
                        max_aniso: f32, wrap_mode: ImageWrap, scale: f32,
                        gamma: f32) -> Arc<MIPMap<Spectrum>> {
    let tex_info = TexInfo {
        filename: filename.clone(),
        do_trilinear: do_trilinear,
        max_aniso: max_aniso,
        wrap: wrap_mode,
        gamma: gamma
    };

    if let Some(tex) = SPECTRUM_TEXTURES.lock().unwrap().get(&tex_info) {
        return tex.clone();
    }

    let read_img_result = read_image(filename);
    let ret = if let Ok((width, height, texels)) = read_img_result {
        // Convert texels to Spectrum and create MIPMap
        let pixels = texels.into_iter()
            .map(|s| (s * scale).powf(gamma))
            .collect();
        Arc::new(MIPMap::new(width, height, pixels))
    } else {
        // Create one-values mipmap
        Arc::new(MIPMap::new(1, 1, vec![Spectrum::from(scale.powf(gamma)); 1]))
    };

    SPECTRUM_TEXTURES.lock().unwrap().insert(tex_info.clone(), ret);
    SPECTRUM_TEXTURES.lock().unwrap().get(&tex_info).unwrap().clone()
}

impl<T> ImageTexture<T> {
    pub fn new_float_texture(m: Box<TextureMapping2D>, filename: &String,
                             do_trilinear: bool, max_aniso: f32,
                             wrap_mode: ImageWrap, scale: f32, gamma: f32)
                             -> ImageTexture<f32> {
        ImageTexture {
            mipmap: get_f32_texture(
                filename, do_trilinear, max_aniso, wrap_mode, scale, gamma),
            mapping: m
        }
    }

    pub fn new_rgb_texture(m: Box<TextureMapping2D>, filename: &String,
                           do_trilinear: bool, max_aniso: f32,
                           wrap_mode: ImageWrap, scale: f32, gamma: f32)
                           -> ImageTexture<Spectrum> {
        ImageTexture {
            mipmap: get_spectrum_texture(
                filename, do_trilinear, max_aniso, wrap_mode, scale, gamma),
            mapping: m
        }
    }
}
