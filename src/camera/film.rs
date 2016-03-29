extern crate image;

use self::image::ImageBuffer;

use camera::CameraSample;
use filter::Filter;
use spectrum::Spectrum;
use utils::Clamp;

use spectrum::xyz_to_rgb;
use utils::get_crop_window;

const FILTER_TABLE_DIM: usize = 16;
const FILTER_TABLE_SIZE: usize = FILTER_TABLE_DIM * FILTER_TABLE_DIM;

fn write_img(filename: &String, rgb: &[f32],
             x_pixel_count: usize, y_pixel_count: usize) {
    assert!(rgb.len() == x_pixel_count * y_pixel_count * 3);

    let img = ImageBuffer::from_fn(
        x_pixel_count as u32, y_pixel_count as u32, |x, y| {
        let idx = 3 * ((y as usize) * x_pixel_count + (x as usize));
        let to_byte = |p: f32| {
            (255.0 * p.powf(1.0 / 2.2) + 0.5).clamp(0.0, 255.0) as u8
        };
        let r = to_byte(rgb[idx + 0]);
        let g = to_byte(rgb[idx + 1]);
        let b = to_byte(rgb[idx + 2]);

        image::Rgb([r, g, b])
    });

    img.save(filename);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pixel {
    xyz: [f32; 3],
    splat_xyz: [f32; 3],
    weight_sum: f32,
    _pad: f32
}

impl Pixel {
    fn new() -> Pixel {
        Pixel {
            xyz: [0.0, 0.0, 0.0],
            weight_sum: 0.0,
            splat_xyz: [0.0, 0.0, 0.0],
            _pad: 0.0
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilmTy {
    Image {
        filter: Filter,
        crop_window: [f32; 4],
        filename: String,
        x_pixel_start: i32,
        y_pixel_start: i32,
        x_pixel_count: usize,
        y_pixel_count: usize,
        pixels: Vec<Pixel>,  // !SPEED! Use a z-ordering here
        filter_table: Vec<f32>
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Film {
    x_res: usize,
    y_res: usize,
    ty: FilmTy
}

impl Film {
    pub fn image(xres: usize, yres: usize, filter: Filter,
                 crop: [f32; 4], filename: String,
                 open_window: bool) -> Film {
        // Compute film image extent
        let x_start = ((xres as f32) * crop[0]).ceil() as i32;
        let x_count = ::std::cmp::max((((xres as f32) * crop[1]).ceil() as i32) - x_start, 1) as usize;
        let y_start = ((yres as f32) * crop[2]).ceil() as i32;
        let y_count = ::std::cmp::max((((yres as f32) * crop[3]).ceil() as i32) - y_start, 1) as usize;

        debug_assert!(x_count > 0);
        debug_assert!(y_count > 0);

        // Precompute filter weight table
        let mut ft = vec![0.0; FILTER_TABLE_SIZE];
        for y in 0..FILTER_TABLE_DIM {
            let fy = ((y as f32) + 0.5) * filter.y_width() / (FILTER_TABLE_DIM as f32);
            for x in 0..FILTER_TABLE_DIM {
                let fx = ((x as f32) + 0.5) * filter.x_width() / (FILTER_TABLE_DIM as f32);
                ft[y * FILTER_TABLE_DIM + x] = filter.evaluate(fx, fy);
            }
        }

        // Possibly open window for image display
        if open_window {
            unimplemented!()
        }

        Film {
            x_res: xres,
            y_res: yres,
            ty: FilmTy::Image {
                filter: filter,
                crop_window: crop,
                filename: filename,
                x_pixel_start: x_start,
                x_pixel_count: x_count,
                y_pixel_start: y_start,
                y_pixel_count: y_count,

                // Allocate film image storage
                pixels: vec![Pixel::new(); x_count * y_count],

                filter_table: ft
            }
        }
    }

    pub fn get_sub_film(&self, num: usize, count: usize) -> Film {
        let aspect = (self.x_res as f32) / (self.y_res as f32);
        let (x0, x1, y0, y1) = get_crop_window(num, count, aspect);

        match &self.ty {
            &FilmTy::Image { ref filter, ref crop_window, ref filename, .. } => {
                // If this isn't our crop window then we need to rethink our
                // design... perhaps construct a new type for subfilms?
                let ox0 = crop_window[0];
                let ox1 = crop_window[1];
                let oy0 = crop_window[2];
                let oy1 = crop_window[3];

                let dx = ox1 - ox0;
                let dy = oy1 - oy0;

                let cw = [ ox0 + x0 * dx, ox0 + x1 * dx,
                           oy0 + y0 * dy, oy0 + y1 * dy ];

                Film::image(self.x_res, self.y_res, filter.clone(),
                            cw, filename.clone(), false)
            }
        }
    }

    fn assign_pixels(&mut self, new_pix: &[Pixel],
                     lx0: i32, lx1: i32, ly0: i32, ly1: i32) {
        let (gx0, gx1, gy0, gy1) = self.get_pixel_extent();

        assert!(lx0 >= gx0);
        assert!(ly0 >= gy0);

        assert!(lx1 <= gx1);
        assert!(ly1 <= gy1);

        match &mut self.ty {
            &mut FilmTy::Image { ref mut pixels, .. } => {
                let lstride = lx1 - lx0;
                let gstride = gx1 - gx0;
                for x in lx0..lx1 {
                    for y in ly0..ly1 {
                        let local_idx = (y - ly0) * lstride + (x - lx0);
                        let global_idx = (y - gy0) * gstride + (x - gx0);
                        pixels[global_idx as usize] =
                            new_pix[local_idx as usize].clone();
                    }
                }
            }
        }
    }

    pub fn add_sub_film(&mut self, f: Film) {
        assert_eq!(f.x_res, self.x_res);
        assert_eq!(f.y_res, self.y_res);

        let (lx0, lx1, ly0, ly1) = f.get_pixel_extent();

        match &f.ty {
            &FilmTy::Image { ref pixels, .. } => {
                self.assign_pixels(&pixels, lx0, lx1, ly0, ly1);
            }
        }
    }

    pub fn x_res(&self) -> usize { self.x_res }
    pub fn y_res(&self) -> usize { self.y_res }

    pub fn num_pixels(&self) -> usize { self.x_res * self.y_res }
    pub fn add_sample(&mut self, sample: &CameraSample, ls: &Spectrum) {
        match &mut self.ty {
            &mut FilmTy::Image { ref filter, x_pixel_start, x_pixel_count,
                                 y_pixel_start, y_pixel_count, ref mut pixels,
                                 ref filter_table, .. } => {
                // Compute sample's raster extent
                let dimage_x = sample.image_x - 0.5;
                let dimage_y = sample.image_y - 0.5;

                let x0 = ::std::cmp::max(x_pixel_start,
                                         (dimage_x - filter.x_width()).ceil() as i32);
                let x1 = ::std::cmp::min(x_pixel_start + (x_pixel_count as i32) - 1,
                                         (dimage_x + filter.x_width()).floor() as i32);
                let y0 = ::std::cmp::max(y_pixel_start,
                                         (dimage_y - filter.y_width()).ceil() as i32);
                let y1 = ::std::cmp::min(y_pixel_start + (y_pixel_count as i32) - 1,
                                         (dimage_y + filter.y_width()).floor() as i32);

                if (x1 - x0) < 0 || (y1 - y0) < 0  { return; }

                // Loop over filter support and add sample to pixel arrays
                let xyz = ls.to_xyz();

                // Precompute x and y filter table offsets
                let ifx = (x0..(x1 + 1)).map(|x| {
                    let fx = ((x as f32) - dimage_x) * filter.inv_x_width() * (FILTER_TABLE_DIM as f32);
                    ::std::cmp::min(fx.abs().floor() as usize, FILTER_TABLE_DIM - 1)
                }).collect::<Vec<_>>();

                let ify = (y0..(y1 + 1)).map(|y| {
                    let fy = ((y as f32) - dimage_y) * filter.inv_y_width() * (FILTER_TABLE_DIM as f32);
                    ::std::cmp::min(fy.abs().floor() as usize, FILTER_TABLE_DIM - 1)
                }).collect::<Vec<_>>();

                for y in y0..(y1 + 1) {
                    for x in x0..(x1 + 1) {
                        // Evaluate filter value at (x, y) pixel
                        let offset = ify[(y-y0) as usize] * FILTER_TABLE_DIM + ifx[(x - x0) as usize];
                        let filter_wt = filter_table[offset];

                        // Update pixel values with filtered sample contribution
                        let pixel_idx = ((y - y_pixel_start) as usize) * x_pixel_count
                            + ((x - x_pixel_start) as usize);
                        let pixel: &mut Pixel = &mut pixels[pixel_idx];

                        // Safely update xyz and weight_sum even with concurrency
                        // !FIXME! These should be atomic once we fix the coarse grained
                        // synchronization granularity after reporting results of samples
                        // in src/sampler_renderer.rs
                        pixel.xyz[0] += filter_wt * xyz[0];
                        pixel.xyz[1] += filter_wt * xyz[1];
                        pixel.xyz[2] += filter_wt * xyz[2];
                        pixel.weight_sum += filter_wt;
                    }
                }
            },
        }
    }

    pub fn splat(&mut self, sample: &CameraSample, ls: &Spectrum) {
        match &mut self.ty {
            &mut FilmTy::Image { x_pixel_start, x_pixel_count,
                                 y_pixel_start, y_pixel_count,
                                 ref mut pixels, .. } => {
                let xyz = ls.to_xyz();
                let x = sample.image_x as i32;
                let y = sample.image_y as i32;

                let (dx, dy) = if x < x_pixel_start || y < y_pixel_start {
                    return;
                } else {
                    ((x - x_pixel_start) as usize, (y - y_pixel_start) as usize)
                };

                if dx >= x_pixel_count || dy >= y_pixel_count { return }

                let pixel_idx = dy * x_pixel_count + dx;
                let pixel: &mut Pixel = &mut pixels[pixel_idx];

                for (i, &x) in xyz.iter().enumerate() {
                    pixel.splat_xyz[i] += x;
                }
            },
        }
    }

    // Returns (x_start, x_end, y_start, y_end) in terms of image space coordinates that
    // should be used to generate samples.
    pub fn get_sample_extent(&self) -> (i32, i32, i32, i32) {
        match &self.ty {
            &FilmTy::Image { x_pixel_start, x_pixel_count,
                             y_pixel_start, y_pixel_count,
                             ref filter, .. } => {
                let x_start = ((x_pixel_start as f32) + 0.5 - filter.x_width()).floor();
                let x_end = ((x_pixel_start as f32) + 0.5 + (x_pixel_count as f32)
                             + filter.x_width()).floor();

                let y_start = ((y_pixel_start as f32) + 0.5 - filter.y_width()).floor();
                let y_end = ((y_pixel_start as f32) + 0.5 + (y_pixel_count as f32)
                             + filter.y_width()).floor();

                (x_start as i32, x_end as i32, y_start as i32, y_end as i32)
            },
        }
    }

    // Returns (x_start, x_end, y_start, y_end) in terms of image space pixel
    // extents. Note: ranges are half-open, so valid pixel coordinates for this
    // film are [x_start, x_end) X [y_start, y_end)
    pub fn get_pixel_extent(&self) -> (i32, i32, i32, i32) {
        match &self.ty {
            &FilmTy::Image { x_pixel_start, x_pixel_count,
                             y_pixel_start, y_pixel_count, .. } => {
                (x_pixel_start, x_pixel_start + x_pixel_count as i32,
                 y_pixel_start, y_pixel_start + y_pixel_count as i32)
            }
        }
    }

    pub fn update_display(&mut self, x0: i32, y0: i32, x1: i32, y1: i32,
                          splat_scale: f32) {
        unimplemented!()
    }

    pub fn write_image(&self, splat_scale: f32) {
        match &self.ty {
            &FilmTy::Image { ref pixels, ref filename,
                             x_pixel_count, y_pixel_count, .. } => {
                // Convert image to RGB and compute final pixel values
                let n_pix = x_pixel_count * y_pixel_count;
                let rgb: Vec<f32> = vec![0.0; n_pix];

                for y in 0..y_pixel_count {
                    for x in 0..x_pixel_count {
                        let offset = y * x_pixel_count + x;

                        let pixel: &Pixel = &pixels[offset];

                        // Convert pixel XYZ color to RGB
                        let mut rgb = xyz_to_rgb(pixel.xyz.clone());

                        // Normalize pixel with weight sum
                        let weight_sum = pixel.weight_sum;
                        if weight_sum != 0.0 {
                            let inv_wt = 1.0 / weight_sum;
                            rgb[0] = (rgb[0] * inv_wt).max(0.0);
                            rgb[1] = (rgb[1] * inv_wt).max(0.0);
                            rgb[2] = (rgb[2] * inv_wt).max(0.0);
                        }

                        // Add splat value at pixel
                        let splat_rgb = xyz_to_rgb(pixel.splat_xyz.clone());
                        rgb[0] = splat_rgb[0] * splat_scale;
                        rgb[1] = splat_rgb[1] * splat_scale;
                        rgb[2] = splat_rgb[2] * splat_scale;
                    }
                }

                // Write RGB image
                write_img(filename, &rgb, x_pixel_count, y_pixel_count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use filter::Filter;

    #[test]
    fn it_can_be_created() {
        let ot = 1.0 / 3.0;
        let tt = 2.0 / 3.0;
        let film = Film::image(142, 12, Filter::mean(3.0, 3.0),
                               [ot, tt, ot, tt], String::from(""), false);

        assert_eq!(film.x_res(), 142);
        assert_eq!(film.y_res(), 12);
        assert_eq!(film.num_pixels(), 142 * 12);
    }

    #[test]
    fn it_has_a_sample_extent() {
        let ot = 1.0 / 3.0;
        let tt = 2.0 / 3.0;
        let film = Film::image(142, 12, Filter::mean(3.0, 3.0),
                               [ot, tt, ot, tt], String::from(""), false);

        let (x0, x1, y0, y1) = film.get_sample_extent();
        assert_eq!(x0, 45); // x-start: 47.333...
        assert_eq!(x1, 98); // x-end: 95.6666
        assert_eq!(y0, 1);  // y-start: 4
        assert_eq!(y1, 11); // y-end: 8

        let adjacent = Film::image(142, 12, Filter::mean(3.0, 3.0),
                                   [0.0, ot, ot, tt], String::from(""), false);
        assert_eq!(adjacent.get_sample_extent(), (-3, 51, 1, 11));
    }

    #[test]
    fn it_has_a_pixel_extent() {
        let ot = 1.0 / 3.0;
        let tt = 2.0 / 3.0;
        let film = Film::image(142, 12, Filter::mean(3.0, 3.0),
                               [ot, tt, ot, tt], String::from(""), false);

        let (x0, x1, y0, y1) = film.get_pixel_extent();
        assert_eq!(x0, 48);     // x-start: 47.333...
        assert_eq!(x1, 95);     // x-end: 95.6666
        assert_eq!(y0, 4);      // y-start: 4
        assert_eq!(y1, 8);      // y-end: 8

        let adjacent = Film::image(142, 12, Filter::mean(3.0, 3.0),
                                   [0.0, ot, ot, tt], String::from(""), false);
        assert_eq!(adjacent.get_pixel_extent(), (0, 48, 4, 8));
    }
}
