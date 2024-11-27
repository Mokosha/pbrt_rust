extern crate lazy_static;
extern crate image;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;
use std::path::Path;
use std::path::PathBuf;
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
use texture::Texture;
use texture::imagewrap::ImageWrap;
use texture::mipmap::MIPMap;
use utils::Lerp;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct TexInfo {
    filename: PathBuf,
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

struct TextureCache<T: std::default::Default + std::clone::Clone>(BTreeMap<TexInfo, Arc<MIPMap<T>>>);

#[derive(Debug)]
pub struct ImageTexture<Tmemory: Default + Clone> {
    mipmap: Arc<MIPMap<Tmemory>>,
    mapping: Box<dyn TextureMapping2D>
}

fn read_image<P>(filename: &P)
                 -> ImageResult<(u32, u32, Vec<Spectrum>)> where P: AsRef<Path> {
    open(filename)
        .and_then(|raw_img| Ok(raw_img.into_rgb8()))
        .and_then(|rgb_img| {
            let mut spec_vec = Vec::new();
            for &pixel in rgb_img.pixels() {
                let r = (pixel[0] as f32) / 255f32;
                let g = (pixel[1] as f32) / 255f32;
                let b = (pixel[2] as f32) / 255f32;
                spec_vec.push(Spectrum::from_rgb([r, g, b]));
            }
            Ok((rgb_img.width(), rgb_img.height(), spec_vec))
        })
}

impl TextureCache<f32> {
    pub fn new() -> TextureCache<f32> { TextureCache(BTreeMap::new()) }

    fn get_texture<P>(
        &mut self, filename: &P, do_trilinear: bool, max_aniso: f32, wrap_mode: ImageWrap,
        scale: f32, gamma: f32)
        -> Arc<MIPMap<f32>> where P: AsRef<Path> + AsRef<OsStr> {
        let tex_info = TexInfo {
            filename: PathBuf::from(filename),
            do_trilinear: do_trilinear,
            max_aniso: max_aniso,
            wrap: wrap_mode,
            gamma: gamma
        };

        if let Some(tex) = self.0.get(&tex_info) {
            return tex.clone();
        }

        let read_img_result = read_image(filename);
        let ret = if let Ok((width, height, texels)) = read_img_result {
            // Convert texels to f32 and create MIPMap
            let pixels = texels.into_iter()
                .map(|s| (s.y() * scale).powf(gamma))
                .collect::<Vec<_>>();
            Arc::new(MIPMap::new(width as usize, height as usize, pixels,
                                do_trilinear, max_aniso, wrap_mode))
        } else {
            // Create one-values mipmap
            Arc::new(MIPMap::new(1, 1, vec![scale.powf(gamma); 1],
                                do_trilinear, max_aniso, wrap_mode))
        };

        self.0.insert(tex_info.clone(), ret);
        self.0.get(&tex_info).unwrap().clone()
    }

    pub fn new_texture<P>(
        &mut self, m: Box<dyn TextureMapping2D>, filename: &P, do_trilinear: bool, max_aniso: f32,
        wrap_mode: ImageWrap, scale: f32, gamma: f32)
        -> ImageTexture<f32> where P: AsRef<Path> + AsRef<OsStr> {
        ImageTexture {
            mipmap: self.get_texture(
                filename, do_trilinear, max_aniso, wrap_mode, scale, gamma),
            mapping: m
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl TextureCache<Spectrum> {
    pub fn new() -> TextureCache<Spectrum> { TextureCache(BTreeMap::new()) }

    fn get_texture<P>(
        &mut self, filename: &P, do_trilinear: bool, max_aniso: f32, wrap_mode: ImageWrap,
        scale: f32, gamma: f32)
        -> Arc<MIPMap<Spectrum>> where P: AsRef<Path> + AsRef<OsStr> {
        let tex_info = TexInfo {
            filename: PathBuf::from(filename),
            do_trilinear: do_trilinear,
            max_aniso: max_aniso,
            wrap: wrap_mode,
            gamma: gamma
        };
    
        if let Some(tex) = self.0.get(&tex_info) {
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
    
        self.0.insert(tex_info.clone(), ret);
        self.0.get(&tex_info).unwrap().clone()
    }

    pub fn new_texture<P>(
        &mut self, m: Box<dyn TextureMapping2D>, filename: &P, do_trilinear: bool, max_aniso: f32,
        wrap_mode: ImageWrap, scale: f32, gamma: f32)
        -> ImageTexture<Spectrum> where P: AsRef<Path> + AsRef<OsStr> {
        ImageTexture {
            mipmap: self.get_texture(
                filename, do_trilinear, max_aniso, wrap_mode, scale, gamma),
            mapping: m
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl<T: Default + Clone + ::std::fmt::Debug +
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use texture::mapping2d::PlanarMapping2D;
    use texture::mapping2d::TextureMapping2D;
    use geometry::point::Point;
    use geometry::vector::Vector;

    #[test]
    fn it_can_create_rgb_textures() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_square.png");
        let mut tex_cache = TextureCache::<Spectrum>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, false, 1.0, ImageWrap::Repeat, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.25, 0.25, 0.0);
        assert_eq!(tex.evaluate(&dg), Spectrum::from_rgb([0.0, 0.0, 0.0]));

        dg.p = Point::new_with(0.75, 0.25, 0.0);
        assert_eq!(tex.evaluate(&dg), Spectrum::from_rgb([1.0, 1.0, 1.0]));

        dg.p = Point::new_with(0.25, 0.75, 0.0);
        assert_eq!(tex.evaluate(&dg), Spectrum::from_rgb([1.0, 1.0, 1.0]));

        dg.p = Point::new_with(0.75, 0.75, 0.0);
        assert_eq!(tex.evaluate(&dg), Spectrum::from_rgb([0.0, 0.0, 0.0]));
    }

    #[test]
    fn it_can_create_float_textures() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_stretched.png");
        let mut tex_cache = TextureCache::<f32>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, false, 1.0, ImageWrap::Repeat, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.25, 0.25, 0.0);
        assert_eq!(tex.evaluate(&dg), 0.0);

        dg.p = Point::new_with(0.75, 0.25, 0.0);
        assert_eq!(tex.evaluate(&dg), 1.0);

        dg.p = Point::new_with(0.25, 0.75, 0.0);
        assert_eq!(tex.evaluate(&dg), 1.0);

        dg.p = Point::new_with(0.75, 0.75, 0.0);
        assert_eq!(tex.evaluate(&dg), 0.0);
    }

    #[test]
    fn it_can_adhere_to_the_repeat_wrap_mode() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_square.png");
        let mut tex_cache = TextureCache::<Spectrum>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, false, 1.0, ImageWrap::Repeat, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.25, 0.25, 0.0);
        let x0 = tex.evaluate(&dg);
        for i in 0..3 {
            for j in 0..3 {
                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x += i as f32;
                dg.p.y += j as f32;
                assert_eq!(x0, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x -= i as f32;
                dg.p.y += j as f32;
                assert_eq!(x0, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x += i as f32;
                dg.p.y -= j as f32;
                assert_eq!(x0, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x -= i as f32;
                dg.p.y -= j as f32;
                assert_eq!(x0, tex.evaluate(&dg));
            }
        }
    }

    #[test]
    fn it_can_adhere_to_the_black_wrap_mode() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_square.png");
        let mut tex_cache = TextureCache::<Spectrum>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, false, 1.0, ImageWrap::Black, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        let black = Spectrum::from_rgb([0.0, 0.0, 0.0]);

        for i in 0..10 {
            for j in 0..10 {
                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x += 1.0 + (i as f32) * 0.1;
                dg.p.y += 1.0 + (j as f32) * 0.1;
                assert_eq!(black, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x -= 1.0 + (i as f32) * 0.1;
                dg.p.y += 1.0 + (j as f32) * 0.1;
                assert_eq!(black, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x += 1.0 + (i as f32) * 0.1;
                dg.p.y -= 1.0 + (j as f32) * 0.1;
                assert_eq!(black, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x -= 1.0 + (i as f32) * 0.1;
                dg.p.y -= 1.0 + (j as f32) * 0.1;
                assert_eq!(black, tex.evaluate(&dg));
            }
        }
    }

    #[test]
    fn it_can_adhere_to_the_clamp_wrap_mode() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_square.png");
        let mut tex_cache = TextureCache::<f32>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, false, 1.0, ImageWrap::Clamp, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        for i in 0..10 {
            for j in 0..10 {
                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x += 1.0 + (i as f32) * 0.1;
                assert!((1.0 - tex.evaluate(&dg)).abs() < 0.0001);

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x -= 1.0 + (i as f32) * 0.1;
                assert_eq!(0.0, tex.evaluate(&dg));

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x += 1.0 + (i as f32) * 0.1;
                dg.p.y -= 1.0 + (j as f32) * 0.1;
                assert!((1.0 - tex.evaluate(&dg)).abs() < 0.0001);

                dg.p = Point::new_with(0.25, 0.25, 0.0);
                dg.p.x -= 1.0 + (i as f32) * 0.1;
                dg.p.y -= 1.0 + (j as f32) * 0.1;
                assert_eq!(0.0, tex.evaluate(&dg));
            }
        }
    }

    #[test]
    fn it_can_do_isotropic_sampling() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_stretched.png");
        let mut tex_cache = TextureCache::<f32>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, true, 1.0, ImageWrap::Clamp, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.51, 0.25, 0.0);
        dg.dpdx = Vector::new_with(0.02, 0.0, 0.0);
        dg.dpdy = Vector::new_with(0.0, 0.02, 0.0);

        assert!((tex.evaluate(&dg) - 0.7).abs() < 0.01);
    }

    #[test]
    fn it_can_do_anisotropic_sampling() {
        let mapping = Box::new(PlanarMapping2D::new());
        let this_file = Path::new(file!());
        let test_file = Path::join(this_file.parent().unwrap(),
                                   "testdata/checkerboard_stretched.png");
        let mut tex_cache = TextureCache::<f32>::new();
        let tex = tex_cache.new_texture(
            mapping, &test_file, false, 100.0, ImageWrap::Clamp, 1.0, 2.2);

        let mut dg = DifferentialGeometry::new();
        dg.p = Point::new_with(0.51, 0.48, 0.0);
        dg.dpdx = Vector::new_with(0.02, 0.0, 0.0);
        dg.dpdy = Vector::new_with(0.0, 0.02, 0.0);

        assert!((tex.evaluate(&dg) - 0.76).abs() < 0.01);

        // Make it anisotropic -- we should get much less black
        dg.dpdy.y = 0.002;
        assert!((tex.evaluate(&dg) - 0.88).abs() < 0.01);
    }
}
