extern crate lazy_static;
extern crate image;

use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::ops::Mul;
use std::iter::Sum;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::Arc;

use self::image::open;
use self::image::ImageResult;

use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::mapping2d::TextureMapping2D;
use texture::internal::TextureBase;
use utils::Clamp;

use utils::sinc_1d;
use utils::modulo;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
enum ImageWrap {
    Repeat,
    Black,
    Clamp
}

struct ResampleWeight {
    first_texel: i32,
    weights: [f32; 4]
}

fn resample_weights(oldres: usize, newres: usize) -> Vec<ResampleWeight> {
    assert!(newres >= oldres);

    let filter_width = 2.0;
    (0..newres).map(|i| {
        let center = ((i as f32) + 0.5) * (oldres as f32) / (newres as f32);
        let first_texel = ((center - filter_width) + 0.5).floor() as i32;
        let mut weights: [f32; 4] = [0.0; 4];

        for j in 0..4 {
            let pos = ((first_texel + j) as f32) + 0.5;
            weights[j as usize] = sinc_1d((pos - center) / filter_width, 2.0);
        }

        let inv_sum_wts = 1.0 / weights.iter().sum::<f32>();
        for w in weights.iter_mut() {
            *w *= inv_sum_wts;
        }

        ResampleWeight { first_texel: first_texel, weights: weights }
    }).collect()
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct MIPMap<T> {
    width: usize,
    height: usize,
    pixels: Vec<T>,
    do_trilinear: bool,
    max_anisotropy: f32,
    wrap_mode: ImageWrap
}

impl<T: Clone + Mul<f32> + Sum<<T as Mul<f32>>::Output>> MIPMap<T> {
    pub fn new(w: usize, h: usize, pixels: Vec<T>, do_tri: bool, max_aniso: f32,
               wm: ImageWrap) -> MIPMap<T> {
        let (width, height, pot_pixels) =
            if !w.is_power_of_two() || !h.is_power_of_two() {
                // Resample image to power-of-two resolution
                let wpot = w.next_power_of_two() as usize;
                let hpot = h.next_power_of_two() as usize;

                let mut new_pixels : Vec<T> =
                    Vec::with_capacity((wpot * hpot) as usize);

                let get_orig = |rswt: &ResampleWeight, j: usize, dim: usize| {
                    let ft = rswt.first_texel + (j as i32);
                    match wm {
                        ImageWrap::Repeat => modulo(ft, dim as i32),
                        ImageWrap::Clamp => ft.clamp(0, (dim-1) as i32),
                        _ => ft
                    }
                };

                // Resample image in s direction
                let s_weights = resample_weights(w as usize, wpot);
                assert_eq!(s_weights.len(), wpot);

                for t in 0..h {
                    for s in 0..wpot {
                        new_pixels.push(
                            s_weights[s].weights.iter()
                                .enumerate()
                                .filter_map(|(j, weight)| {
                                    let orig_s = get_orig(&(s_weights[s]), j, w);
                                    if orig_s >= 0 && orig_s < (w as i32) {
                                        let idx = t*h + (orig_s as usize);
                                        Some(pixels[idx].clone() * *weight)
                                    } else {
                                        None
                                    }
                                })
                                .sum());
                    }
                }

                // Resample image in t direction
                let t_weights = resample_weights(h as usize, hpot);
                for s in 0..wpot {
                    for t in 0..hpot {
                        let pixel =
                            t_weights[t].weights.iter()
                            .enumerate()
                            .filter_map(|(j, weight)| {
                                let orig_t = get_orig(&(t_weights[t]), j, h);
                                if orig_t >= 0 && orig_t < (h as i32) {
                                    let idx = (orig_t as usize)*h + s;
                                    Some(new_pixels[idx].clone() * *weight)
                                } else {
                                    None
                                }
                            })
                            .sum();

                        if t >= new_pixels.len() {
                            assert_eq!(new_pixels.len(), t);
                            new_pixels.push(pixel);
                        } else {
                            new_pixels[t * wpot + s] = pixel;
                        }
                    }
                }

                (wpot, hpot, new_pixels)
            } else {
                (w, h, pixels)
            };

        MIPMap {
            width: w,
            height: h,
            pixels: pot_pixels,
            do_trilinear: do_tri,
            max_anisotropy: max_aniso,
            wrap_mode: wm
        }
    }

    pub fn lookup(&self, s: f32, t: f32,
                  dsdx: f32, dtdx: f32,
                  dsdy: f32, dtdy: f32) -> T where T : Clone {
        self.pixels[0].clone()
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
        Arc::new(MIPMap::new(width as usize, height as usize, pixels,
                             do_trilinear, max_aniso, wrap_mode))
    } else {
        // Create one-values mipmap
        Arc::new(MIPMap::new(1, 1, vec![scale.powf(gamma); 1],
                             do_trilinear, max_aniso, wrap_mode))
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
        Arc::new(MIPMap::new(width as usize, height as usize, pixels,
                             do_trilinear, max_aniso, wrap_mode))
    } else {
        // Create one-values mipmap
        Arc::new(MIPMap::new(1, 1, vec![Spectrum::from(scale.powf(gamma)); 1],
                             do_trilinear, max_aniso, wrap_mode))
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

    pub fn clear_cache() {
        FLOAT_TEXTURES.lock().unwrap().clear();
        SPECTRUM_TEXTURES.lock().unwrap().clear();
    }
}

impl super::internal::TextureBase<f32> for ImageTexture<f32> {
    fn eval(&self, dg: &DifferentialGeometry) -> f32 {
        let (s, t, dsdx, dtdx, dsdy, dtdy) = self.mapping.map(dg);
        self.mipmap.lookup(s, t, dsdx, dtdx, dsdy, dtdy)
    }
}

impl super::internal::TextureBase<Spectrum> for ImageTexture<Spectrum> {
    fn eval(&self, dg: &DifferentialGeometry) -> Spectrum {
        let (s, t, dsdx, dtdx, dsdy, dtdy) = self.mapping.map(dg);
        let ret = self.mipmap.lookup(s, t, dsdx, dtdx, dsdy, dtdy);
        Spectrum::from_rgb(ret.to_rgb())
    }
}
