extern crate lazy_static;
extern crate image;

use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;
use std::iter::Sum;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::Arc;

use std::cmp;

use self::image::open;
use self::image::ImageResult;

use diff_geom::DifferentialGeometry;
use spectrum::Spectrum;
use texture::mapping2d::TextureMapping2D;
use texture::internal::TextureBase;
use utils::Clamp;
use utils::Lerp;
use utils::blocked_vec::BlockedVec;

use utils::sinc_1d;
use utils::modulo;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum ImageWrap {
    Repeat,
    Black,
    Clamp
}

struct ResampleWeight {
    first_texel: i32,
    weights: [f32; 4]
}

const INV_EXP_2: f32 = 0.13533528323;

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

fn resize_to_power_of_two_dims<T>
    (w: usize, h: usize, pixels: Vec<T>, wm: ImageWrap) -> (usize, usize, Vec<T>)
    where T : Clone + Mul<f32> + Sum<<T as Mul<f32>>::Output>
{
    // Resample image to power-of-two resolution
    let wpot = w.next_power_of_two();
    let hpot = h.next_power_of_two();

    let mut new_pixels : Vec<T> = Vec::with_capacity(wpot * hpot);

    let get_orig = |rswt: &ResampleWeight, j: usize, dim: usize| {
        let ft = rswt.first_texel + (j as i32);
        let orig = match wm {
            ImageWrap::Repeat => modulo(ft, dim as i32),
            ImageWrap::Clamp => ft.clamp(0, (dim-1) as i32),
            _ => ft
        };

        if orig >= 0 && orig < (dim as i32) { Some(orig) } else { None }
    };

    // Resample image in s direction
    let s_weights = resample_weights(w as usize, wpot);
    assert_eq!(s_weights.len(), wpot);

    for t in 0..h {
        for s in 0..wpot {
            new_pixels.push(
                s_weights[s].weights.iter()
                    .enumerate()
                    .filter_map(
                        |(j, weight)|
                        get_orig(&(s_weights[s]), j, w).map(|orig_s| {
                            pixels[t*h + (orig_s as usize)].clone() * *weight
                        }))
                    .sum());
        }
    }

    // Add the remaining rows
    for t in h..hpot {
        for s in 0..wpot {
            new_pixels.push(pixels[0].clone());
        }
    }

    // Resample image in t direction
    let t_weights = resample_weights(h as usize, hpot);
    for s in 0..wpot {
        for t in 0..hpot {
            new_pixels[t * wpot + s] = t_weights[t].weights.iter()
                .enumerate()
                .filter_map(
                    |(j, weight)|
                    get_orig(&(t_weights[t]), j, h).map(|orig_t| {
                        new_pixels[(orig_t as usize)*h + s].clone() * *weight
                    }))
                .sum();
        }
    }

    (wpot, hpot, new_pixels)
}

fn ulog2(x: usize) -> usize {
    ((0usize).leading_zeros() - x.leading_zeros()) as usize
}

fn texel_at<T>(level: &BlockedVec<T>, _s: i32, _t: i32, wm: ImageWrap)
               -> T where T: Clone + Default {
    // Compute texel (s, t) accounting for boundary conditions
    let (s, t) = match wm {
        ImageWrap::Repeat => (modulo(_s, level.width() as i32),
                              modulo(_t, level.height() as i32)),
        ImageWrap::Clamp => (_s.clamp(0, (level.width() - 1) as i32),
                             _t.clamp(0, (level.height() - 1) as i32)),
        ImageWrap::Black => {
            let out_of_bounds =
                _s < 0 || _s >= (level.width() as i32) ||
                _t < 0 || _t >= (level.height() as i32);
            if out_of_bounds {
                return Default::default();
            } else {
                (_s, _t)
            }
        }
    };

    assert!(s >= 0);  assert!(s < (level.width() as i32));
    assert!(t >= 0);  assert!(t < (level.height() as i32));

    level.get(s as usize, t as usize).unwrap().clone()
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct MIPMap<T: Default + Clone> {
    width: usize,
    height: usize,
    pyramid: Vec<BlockedVec<T>>,
    do_trilinear: bool,
    max_anisotropy: f32,
    wrap_mode: ImageWrap
}

impl<T: Default + Clone +
     Mul<f32, Output = T> +
     Div<f32, Output = T> +
     Sum<<T as Mul<f32>>::Output> +
     Add<Output = T> +
     Lerp<f32>> MIPMap<T> {
    fn new(w: usize, h: usize, pixels: Vec<T>, do_tri: bool, max_aniso: f32,
           wm: ImageWrap) -> MIPMap<T> {
        let (width, height, pot_pixels) =
            if !w.is_power_of_two() || !h.is_power_of_two() {
                resize_to_power_of_two_dims(w, h, pixels, wm)
            } else {
                (w, h, pixels)
            };

        let mut pyramid = Vec::new();
        // Initialize most detailed level of mip-map
        pyramid.push(BlockedVec::new_with(width, height, pot_pixels));

        let num_levels = ulog2(cmp::max(width, height));
        for i in 1..num_levels {
            // Initialize (i-1)st level of the pyramid
            let new_level = {
                let last_level = pyramid.last().unwrap();
                let level_width = cmp::max(last_level.width() / 2, 1);
                let level_height = cmp::max(last_level.height() / 2, 1);

                let mut new_level = BlockedVec::new(level_width, level_height);
                for t in 0..(level_height as i32) {
                    for s in 0..(level_width as i32) {
                        // Filter four pixels from inner level of pyramid
                        let t0 = texel_at(last_level, 2 * s, 2 * t, wm);
                        let t1 = texel_at(last_level, 2 * s + 1, 2 * t, wm);
                        let t2 = texel_at(last_level, 2 * s, 2 * t + 1, wm);
                        let t3 = texel_at(last_level, 2 * s + 1, 2 * t + 1, wm);

                        *(new_level.get_mut(s as usize, t as usize).unwrap()) =
                            (t0 + t1 + t2 + t3) * 0.25;
                    }
                }

                new_level
            };
            pyramid.push(new_level);
        }

        MIPMap {
            width: width,
            height: height,
            pyramid: pyramid,
            do_trilinear: do_tri,
            max_anisotropy: max_aniso,
            wrap_mode: wm
        }
    }

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }

    fn levels(&self) -> usize { self.pyramid.len() }

    fn triangle(&self, _level: usize, _s: f32, _t: f32) -> T {
        let level = _level.clamp(0, self.levels() - 1);
        let s = _s * (self.pyramid[level].width() as f32) - 0.5;
        let t = _t * (self.pyramid[level].height() as f32) - 0.5;
        let s0 = s.floor() as i32;
        let t0 = t.floor() as i32;
        let ds = s - (s0 as f32);
        let dt = t - (t0 as f32);
        let wm = self.wrap_mode;
        texel_at(&self.pyramid[level], s0, t0, wm) * (1.0 - ds) * (1.0 - dt) +
        texel_at(&self.pyramid[level], s0, t0 + 1, wm) * (1.0 - ds) * dt +
        texel_at(&self.pyramid[level], s0 + 1, t0, wm) * ds * (1.0 - dt) +
        texel_at(&self.pyramid[level], s0 + 1, t0 + 1, wm) * ds * dt
    }

    fn pyramid_lookup(&self, s: f32, t: f32, width: f32) -> T {
        let level = (self.levels() as f32) - 1.0 + width.max(1e-8).log2();
        if level < 0.0 {
            self.triangle(0, s, t)
        } else if level >= ((self.levels() - 1) as f32) {
            texel_at(self.pyramid.last().unwrap(), 0, 0, self.wrap_mode)
        } else {
            let ilevel = level as usize;
            let delta = level - (ilevel as f32);
            let t0 = self.triangle(ilevel + 1, s, t);
            let t1 = self.triangle(ilevel, s, t);
            t0.lerp_with(t1, delta)
        }
    }

    fn ewa(&self, level: usize, _s: f32, _t: f32,
           _ds0: f32, _dt0: f32, _ds1: f32, _dt1: f32) -> T {
        if level >= self.levels() {
            return texel_at(&self.pyramid.last().unwrap(), 0, 0, self.wrap_mode);
        }

        // Convert EWA coordinates to appropriate scale for level
        let s = _s * (self.pyramid[level].width() as f32) - 0.5;
        let t = _t * (self.pyramid[level].height() as f32) - 0.5;
        let ds0 = _ds0 * (self.pyramid[level].width() as f32);
        let dt0 = _dt0 * (self.pyramid[level].height() as f32);
        let ds1 = _ds1 * (self.pyramid[level].width() as f32);
        let dt1 = _dt1 * (self.pyramid[level].height() as f32);

        // Compute ellipse coefficients to bound EWA filter region
        let (a, b, c) = {
            let sq = dt0*dt0 + dt1*dt1;
            let a = sq + 1.0;
            let b = -2.0 * sq;
            let c = sq + 1.0;
            let inv_f = 1.0 / (a * c - b * b * 0.25);
            (a * inv_f, b * inv_f, c * inv_f)
        };

        // Compute the ellipse's (s, t) bounding box in texture space
        let det = -b * b + 4.0 * a * c;
        let inv_det = 1.0 / det;
        let u_sqrt = (det * c).sqrt();
        let v_sqrt = (det * a).sqrt();

        let s0 = (s - 2.0 * inv_det * u_sqrt).ceil() as i32;
        let s1 = (s + 2.0 * inv_det * u_sqrt).floor() as i32;
        let t0 = (t - 2.0 * inv_det * v_sqrt).ceil() as i32;
        let t1 = (t + 2.0 * inv_det * v_sqrt).floor() as i32;

        // Scan over ellipse bound and compute quadratic equation
        let (sum, sum_wts): (T, f32) =
            (t0..(t1 + 1)).fold((Default::default(), 0.0), |acc, it| {
                let tt = (it as f32) - t;
                (s0..(s0 + 1)).fold(acc, |(sum, wts), is| {
                    let ss = (is as f32) - s;
                    // Compute squared radius and filter texel if inside ellipse
                    let r2 = a * ss * ss + b * ss * tt + c * tt * tt;
                    if r2 < 1.0 {
                        // !SPEED! This is a LUT in the book, but for now we
                        // can just leave it as-is here.
                        let weight = (-2.0 * r2).exp() - INV_EXP_2;
                        let wm = self.wrap_mode;
                        (sum + texel_at(&self.pyramid[level], is, it, wm) * weight,
                         wts + weight)
                    } else {
                        (sum, wts)
                    }
                })
            });

        sum / sum_wts
    }

    fn lookup(&self, s: f32, t: f32,
              dsdx: f32, dtdx: f32, dsdy: f32, dtdy: f32) -> T {
        if self.do_trilinear {
            let width =
                dsdx.abs().max(dtdx.abs()).max(dsdy.abs()).max(dtdy.abs());
            return self.pyramid_lookup(s, t, 2.0 * width);
        }

        // Compute ellipse minor and major axes
        let (ds0, dt0, ds1, dt1) =
            if dsdx * dsdx + dtdx * dtdx > dsdy * dsdy + dtdy * dtdy {
                (dsdx, dtdx, dsdy, dtdy)
            } else {
                (dsdy, dtdy, dsdx, dtdx)
            };
        let major_length = (ds0*ds0 + dt0*dt0).sqrt();
        let minor_length = (ds1*ds1 + dt1*dt1).sqrt();

        // Clamp ellipse eccentricity if too large
        let max_major_length = minor_length * self.max_anisotropy;
        let (scaled_ds1, scaled_dt1, scaled_minor_length) =
            if max_major_length < major_length && minor_length > 0.0 {
                let scale = major_length / (minor_length * self.max_anisotropy);
                (ds1 * scale, dt1 * scale, minor_length * scale)
            } else {
                (ds1, dt1, minor_length)
            };

        if scaled_minor_length == 0.0 {
            return self.triangle(0, s, t);
        }

        // Choose level of detail for EWA lookup and perform EWA filtering.
        let lod = ((self.levels() as f32) - 1.0 + minor_length.log2()).max(0.0);
        let ilod = lod.floor() as usize;
        let d = lod - (ilod as f32);
        let t0 = self.ewa(ilod + 0, s, t, ds0, dt0, scaled_ds1, scaled_dt1);
        let t1 = self.ewa(ilod + 1, s, t, ds0, dt0, scaled_ds1, scaled_dt1);
        t0.lerp_with(t1, d)
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

struct ImageTexture<Tmemory: Default + Clone> {
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

impl<T: Default + Clone> ImageTexture<T> {
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

impl<T: Default + Clone +
     Mul<f32, Output = T> +
     Div<f32, Output = T> +
     Sum<<T as Mul<f32>>::Output> +
     Add<Output = T> +
     Lerp<f32>>
super::internal::TextureBase<T> for ImageTexture<T> {
    fn eval(&self, dg: &DifferentialGeometry) -> T {
        let (s, t, dsdx, dtdx, dsdy, dtdy) = self.mapping.map(dg);
        self.mipmap.lookup(s, t, dsdx, dtdx, dsdy, dtdy)
    }
}
