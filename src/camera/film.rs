use camera::CameraSample;
use filter::Filter;
use spectrum::Spectrum;

const FILTER_TABLE_DIM: usize = 16;
const FILTER_TABLE_SIZE: usize = FILTER_TABLE_DIM * FILTER_TABLE_DIM;

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
    }
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
    }

    pub fn get_sample_extent(&self) -> (i32, i32, i32, i32) {
        (0, 0, 0, 0)
    }

    pub fn get_pixel_extent(&self) -> (i32, i32, i32, i32) {
        (0, 0, 0, 0)
    }

    pub fn update_display(&mut self, x0: i32, y0: i32, x1: i32, y1: i32,
                          splat_scale: f32) {
    }

    pub fn write_image(&mut self, splat_scale: f32) {
    }
}
